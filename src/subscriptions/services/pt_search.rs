use futures_util::future::join_all;
use tokimo_pt_search::{PtSearchResult, SiteAuth, SiteType, get_site_config, search_site};

use crate::shared::categories::{Category, resolve_category};
use crate::subscriptions::models::pt_site::PtSiteDto;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PtSearchResponse {
    pub results: Vec<PtSearchResultWithSite>,
    pub site_summaries: Vec<SiteSummary>,
    pub total: usize,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PtSearchResultWithSite {
    #[serde(flatten)]
    pub result: PtSearchResult,
    pub site_db_id: String,
    pub site_name: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteSummary {
    pub site_db_id: String,
    pub site_name: String,
    pub count: usize,
}

/// Fix detail_url when it's just a raw ID (common for NexusPHP sites).
/// For API sites, set download_url to genDlToken endpoint.
/// Uses `detail_url_template` from site config if available.
fn fix_urls(result: &mut PtSearchResult, domain: &str, site_id: &str) {
    let config = get_site_config(site_id);

    // Fix detail_url — if site has a template, always use it (API URLs are not human-readable)
    if let Some(template) = config.as_ref().and_then(|c| c.detail_url_template) {
        // Extract ID from current detail_url if it's a full URL
        let id = extract_id_from_url(&result.detail_url).or_else(|| Some(result.id.clone()));
        if let Some(tid) = id {
            result.detail_url = template.replace("{id}", &tid);
        }
    } else if !result.detail_url.is_empty()
        && !result.detail_url.starts_with("http")
        && !result.detail_url.contains('/')
        && !result.detail_url.contains('?')
    {
        // Default NexusPHP format
        let base = domain.trim_end_matches('/');
        result.detail_url = format!("{}/details.php?id={}", base, result.detail_url);
    }

    // Fix download_url
    if !result.download_url.is_empty()
        && !result.download_url.starts_with("http")
        && !result.download_url.contains('/')
        && !result.download_url.contains('?')
    {
        let is_api_site = config.as_ref().is_some_and(|c| c.site_type == SiteType::Api);

        let base = domain.trim_end_matches('/');
        if is_api_site {
            result.download_url = format!("{base}/api/torrent/genDlToken");
        } else {
            result.download_url = format!("{}/download.php?id={}", base, result.download_url);
        }
    }
}

/// Extract torrent ID from a URL like `https://api.m-team.cc/details.php?id=673722`
fn extract_id_from_url(url: &str) -> Option<String> {
    if url.is_empty() {
        return None;
    }
    // Try query param: ?id=XXX
    if let Some(pos) = url.find("id=") {
        let after = &url[pos + 3..];
        let id = after.split('&').next().unwrap_or(after);
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }
    // Try path segment: /detail/XXX
    if let Some(pos) = url.find("/detail/") {
        let after = &url[pos + 8..];
        let id = after
            .split('/')
            .next()
            .unwrap_or(after)
            .split('?')
            .next()
            .unwrap_or(after);
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }
    // If it's just a raw ID
    if !url.contains('/') && !url.contains('?') {
        return Some(url.to_string());
    }
    None
}

pub async fn search_all_sites(
    http_client: &reqwest::Client,
    sites: &[PtSiteDto],
    keyword: &str,
    filter_categories: &[String],
) -> PtSearchResponse {
    // Normalize filter categories to canonical slugs for matching
    let slug_filters: Vec<String> = filter_categories
        .iter()
        .filter_map(|c| Category::from_slug(c).map(|cat| cat.slug().to_string()))
        .collect();

    let futures: Vec<_> = sites
        .iter()
        .map(|site| search_single_site(http_client, site, keyword))
        .collect();

    let results_per_site = join_all(futures).await;

    let mut all_results = Vec::new();
    let mut site_summaries = Vec::new();

    for (site, results) in sites.iter().zip(results_per_site) {
        let mut count = 0;
        for mut result in results {
            fix_urls(&mut result, &site.domain, &site.site_id);

            // Resolve raw site category to a canonical slug
            let slug = resolve_category(&site.site_id, &result.category);

            // Apply category filter (match against canonical slug)
            if !slug_filters.is_empty() && !slug_filters.contains(&slug) {
                continue;
            }

            // Replace the raw site category with the canonical slug
            result.category = slug;

            count += 1;
            all_results.push(PtSearchResultWithSite {
                result,
                site_db_id: site.id.clone(),
                site_name: site.name.clone(),
            });
        }
        site_summaries.push(SiteSummary {
            site_db_id: site.id.clone(),
            site_name: site.name.clone(),
            count,
        });
    }

    let total = all_results.len();

    // Sort by seeders descending
    all_results.sort_by_key(|a| std::cmp::Reverse(a.result.seeders));

    PtSearchResponse {
        results: all_results,
        site_summaries,
        total,
    }
}

async fn search_single_site(http_client: &reqwest::Client, site: &PtSiteDto, keyword: &str) -> Vec<PtSearchResult> {
    let auth = SiteAuth {
        cookies: site.cookies.clone(),
        api_key: site.api_key.clone(),
    };

    search_site(
        http_client,
        &site.site_id,
        keyword,
        &site.domain,
        &auth,
        site.adult_enabled,
    )
    .await
}
