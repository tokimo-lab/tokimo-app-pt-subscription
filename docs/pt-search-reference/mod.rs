pub mod api_site;
pub mod config;
pub mod nexus;
pub mod parser;
pub mod sites;

use serde::{Deserialize, Serialize};
use tracing::warn;

use config::SiteType;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtSearchResult {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub size: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    pub seeders: i32,
    pub leechers: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grabs: Option<i32>,
    pub category: String,
    pub upload_time: String,
    pub download_url: String,
    pub detail_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_rating: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub douban_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub douban_rating: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_codec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_codec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

pub struct SiteAuth {
    pub cookies: Option<String>,
    pub api_key: Option<String>,
}

/// Search a specific PT site for torrents.
///
/// Returns an empty `Vec` if the site is unknown or an error occurs.
pub async fn search_site(
    client: &reqwest::Client,
    site_id: &str,
    keyword: &str,
    domain: &str,
    auth: &SiteAuth,
    adult_enabled: bool,
) -> Vec<PtSearchResult> {
    let Some(config) = sites::get_site_config(site_id) else {
        warn!(site_id, "Unknown PT site");
        return Vec::new();
    };

    let mut results = match config.site_type {
        SiteType::NexusPhp | SiteType::Public => nexus::search(client, keyword, domain, auth, &config).await,
        SiteType::Api => api_site::search(client, keyword, domain, auth, &config).await,
    };

    // Optionally search adult content
    if adult_enabled && let Some(ref adult_search) = config.adult_search {
        let adult_config = config::SiteConfig {
            search: adult_search.clone(),
            ..config.clone()
        };
        let adult_results = match config.site_type {
            SiteType::Api => api_site::search(client, keyword, domain, auth, &adult_config).await,
            _ => nexus::search(client, keyword, domain, auth, &adult_config).await,
        };
        results.extend(adult_results);
    }

    results
}
