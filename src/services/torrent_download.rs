//! Reusable PT torrent download service.
//!
//! Owns the full "fetch .torrent → parse → add to client (with optional
//! episode filtering)" flow so both the HTTP handler and the CLI can drive a
//! one-shot download without depending on the full `AppState`.

use sea_orm::DatabaseConnection;
use tokimo_package_client_api::downloaders::traits::AddTorrentOptions;

use crate::AppError;
use crate::db::repos::download_client_repo::DownloadClientRepo;
use crate::services::DownloadClientService;
use crate::shared::episode_parser::should_include_file;
use crate::shared::path::{normalize_category_slug, resolve_download_path};
use crate::shared::torrent_parser::parse_torrent;
use crate::subscriptions::repos::pt_site_repo::PtSiteRepo;

/// Parameters for [`download_torrent_to_client`].
pub struct DownloadTorrentParams {
    /// PT site DB id (`PtSiteDto.id`).
    pub site_id: String,
    /// Site-local torrent id (`PtSearchResult.id`).
    pub torrent_id: String,
    /// Download client DB id.
    pub client_id: String,
    /// Canonical category slug (used for save-path resolution + client category).
    pub category: Option<String>,
    /// Explicit save path override; resolved from client paths when `None`.
    pub save_path: Option<String>,
    /// Optional season filter (TV).
    pub season: Option<i32>,
    /// Optional episode filter (TV).
    pub episodes: Option<Vec<i32>>,
    /// Optional client-side tags.
    pub tags: Option<Vec<String>>,
    /// Add in paused state (only honored when no episode filtering is needed,
    /// since filtering must add paused then resume after setting priorities).
    pub paused: Option<bool>,
}

/// Result of a successful download dispatch.
pub struct DownloadOutcome {
    /// Parsed torrent name (as registered in the download client).
    pub torrent_name: String,
    /// Total number of files in the torrent.
    pub total_files: usize,
    /// Number of files excluded by the episode filter.
    pub excluded_files: usize,
    /// The resolved save path the torrent was added with (if any).
    pub save_path: Option<String>,
}

/// Download .torrent file bytes. Handles M-Team's genDlToken API flow:
/// if the URL contains "genDlToken", POST to get the real download URL first.
pub async fn download_torrent_bytes(
    http_client: &reqwest::Client,
    download_url: &str,
    api_key: Option<&str>,
    torrent_id: Option<&str>,
) -> Result<bytes::Bytes, AppError> {
    // M-Team API: need to call genDlToken first to get actual download URL
    if download_url.contains("genDlToken") || download_url.contains("/api/") {
        let no_redirect_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_else(|_| http_client.clone());

        let form_body = match torrent_id {
            Some(tid) => format!("id={tid}"),
            None => String::new(),
        };

        let mut req = no_redirect_client
            .post(download_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_body);
        if let Some(key) = api_key {
            req = req.header("x-api-key", key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::bad_request(format!("调用 genDlToken 失败: {e}")))?;

        let status = resp.status();
        if !status.is_success() && !status.is_redirection() {
            return Err(AppError::bad_request(format!("genDlToken 失败: HTTP {status}")));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::bad_request(format!("解析 genDlToken 响应失败: {e}")))?;

        let real_url = json
            .get("data")
            .and_then(|d| d.as_str())
            .ok_or_else(|| AppError::bad_request("genDlToken 未返回下载链接"))?;

        // Download the actual .torrent file (needs follow redirects)
        let dl_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(5))
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_else(|_| http_client.clone());

        let torrent_resp = dl_client
            .get(real_url)
            .send()
            .await
            .map_err(|e| AppError::bad_request(format!("下载种子文件失败: {e}")))?;

        if !torrent_resp.status().is_success() {
            return Err(AppError::bad_request(format!(
                "下载种子文件失败: HTTP {}",
                torrent_resp.status()
            )));
        }

        torrent_resp
            .bytes()
            .await
            .map_err(|e| AppError::bad_request(format!("读取种子文件失败: {e}")))
    } else {
        // Direct .torrent URL
        let no_redirect_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_else(|_| http_client.clone());

        let mut req = no_redirect_client.get(download_url);
        if let Some(key) = api_key {
            req = req.header("x-api-key", key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::bad_request(format!("下载种子文件失败: {e}")))?;

        if !resp.status().is_success() && !resp.status().is_redirection() {
            return Err(AppError::bad_request(format!(
                "下载种子文件失败: HTTP {}",
                resp.status()
            )));
        }

        resp.bytes()
            .await
            .map_err(|e| AppError::bad_request(format!("读取种子文件失败: {e}")))
    }
}

/// Build the site-specific download URL + torrent id payload for a given site.
fn build_download_url(domain: &str, site_id: &str, torrent_id: &str) -> (String, Option<String>) {
    let site_config = tokimo_pt_search::get_site_config(site_id);
    let is_api = site_config
        .as_ref()
        .is_some_and(|c| c.site_type == tokimo_pt_search::SiteType::Api);
    let domain = domain.trim_end_matches('/');

    if is_api {
        (format!("{domain}/api/torrent/genDlToken"), Some(torrent_id.to_string()))
    } else {
        (format!("{domain}/download.php?id={torrent_id}"), None)
    }
}

/// Fetch a PT torrent and add it to a download client, optionally filtering to a
/// specific season/episode set. Depends only on `db` + `http_client` so both the
/// HTTP handler and the CLI can call it.
#[allow(clippy::too_many_lines)]
pub async fn download_torrent_to_client(
    db: &DatabaseConnection,
    http_client: &reqwest::Client,
    params: DownloadTorrentParams,
) -> Result<DownloadOutcome, AppError> {
    // Look up site config
    let site = PtSiteRepo::get_by_id(db, &params.site_id)
        .await?
        .ok_or_else(|| AppError::not_found("站点不存在"))?;

    let (download_url, dl_torrent_id) = build_download_url(&site.domain, &site.site_id, &params.torrent_id);

    // 1. Download .torrent file (handles genDlToken for API sites)
    let bytes = download_torrent_bytes(
        http_client,
        &download_url,
        site.api_key.as_deref(),
        dl_torrent_id.as_deref(),
    )
    .await?;
    let meta = parse_torrent(&bytes).map_err(AppError::bad_request)?;

    // 2. Determine which files to exclude based on episode filter
    let filter_season = params.season;
    let filter_episodes = params.episodes.as_deref().unwrap_or(&[]);
    let has_filter = filter_season.is_some() || !filter_episodes.is_empty();

    let excluded_indices: Vec<u32> = if has_filter {
        meta.files
            .iter()
            .filter(|f| !should_include_file(&f.path, filter_season, filter_episodes))
            .map(|f| f.index as u32)
            .collect()
    } else {
        vec![]
    };

    // 3. Resolve save path if not explicitly provided
    let save_path = if params.save_path.is_some() {
        params.save_path.clone()
    } else {
        let client = DownloadClientRepo::get_by_id(db, &params.client_id)
            .await?
            .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;
        let paths: Vec<(String, String, String)> = client
            .download_paths
            .iter()
            .map(|p| (p.r#type.clone(), p.path.clone(), p.description.clone()))
            .collect();
        let cat_raw = params.category.as_deref().unwrap_or("global");
        resolve_download_path(&paths, &normalize_category_slug(cat_raw))
    };

    // 4. Add torrent to download client (paused if we need to filter files)
    let need_filter = !excluded_indices.is_empty();
    let requested_paused = params.paused.unwrap_or(false);
    let torrent_bytes = base64::Engine::encode(&base64::engine::general_purpose::STANDARD_NO_PAD, &bytes);

    let options = AddTorrentOptions {
        urls: None,
        torrents: Some(vec![torrent_bytes]),
        save_path: save_path.clone(),
        category: params.category.clone(),
        tags: params.tags.clone(),
        paused: Some(need_filter || requested_paused),
        skip_hash_check: None,
    };

    DownloadClientService::add_torrent(db, &params.client_id, options, http_client).await?;

    // 5. If we need to filter, find the torrent by name and set file priorities
    if need_filter {
        // Wait briefly for qBittorrent to register the torrent
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Find the torrent by name
        let torrents = DownloadClientService::get_torrents(db, &params.client_id, None, None, http_client).await?;

        let target = torrents.iter().find(|t| t.name == meta.name);
        if let Some(torrent) = target {
            // Set excluded files to priority 0 (don't download)
            let hash = &torrent.hash;
            DownloadClientService::set_file_priority(
                db,
                &params.client_id,
                hash,
                &excluded_indices,
                0, // don't download
                http_client,
            )
            .await?;

            // Resume the torrent unless the caller explicitly wanted it paused
            if !requested_paused {
                DownloadClientService::resume_torrents(db, &params.client_id, std::slice::from_ref(hash), http_client)
                    .await?;
            }

            tracing::info!(
                "Torrent added with filter: excluded {} of {} files",
                excluded_indices.len(),
                meta.files.len()
            );
        } else {
            tracing::warn!(
                "Could not find torrent by name '{}', resuming without filter",
                meta.name
            );
        }
    }

    Ok(DownloadOutcome {
        torrent_name: meta.name,
        total_files: meta.files.len(),
        excluded_files: excluded_indices.len(),
        save_path,
    })
}
