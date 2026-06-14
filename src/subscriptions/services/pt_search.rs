use futures_util::future::join_all;
use tokimo_pt_search::{PtSearchResult, SiteAuth, SiteType, get_site_config, search_site};

use crate::shared::categories::{resolve_category, resolve_from_str};
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
    /// Canonical category name in English (e.g. "music", "movie")
    pub category_name: String,
    /// Canonical category display name in Chinese (e.g. "音乐", "电影")
    pub category_display_name: String,
    /// Canonical category ID
    pub canonical_category_id: String,
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
        let is_api_site = config.as_ref().map(|c| c.site_type == SiteType::Api).unwrap_or(false);

        let base = domain.trim_end_matches('/');
        if is_api_site {
            result.download_url = format!("{}/api/torrent/genDlToken", base);
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
    // Convert filter categories to canonical category IDs for matching
    let canonical_filters: Vec<String> = filter_categories
        .iter()
        .filter_map(|c| resolve_from_str(c).map(|cat| cat.id().to_string()))
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

            // Resolve category using static mapping
            let (cat_canonical, cat_name, cat_display) = resolve_category(&site.site_id, &result.category);

            // Apply category filter (match against canonical ID)
            if !canonical_filters.is_empty() && !canonical_filters.contains(&cat_canonical) {
                continue;
            }

            count += 1;
            all_results.push(PtSearchResultWithSite {
                result,
                site_db_id: site.id.clone(),
                site_name: site.name.clone(),
                category_name: cat_name,
                category_display_name: cat_display,
                canonical_category_id: cat_canonical,
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
    all_results.sort_by(|a, b| b.result.seeders.cmp(&a.result.seeders));

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
