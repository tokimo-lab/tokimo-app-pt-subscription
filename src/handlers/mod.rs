pub mod user;

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use tokimo_package_client_api::downloaders::traits::AddTorrentOptions;

use crate::AppError;
use crate::AppState;
use crate::db::repos::download_client_repo::{
    CreateDownloadClientInput, DownloadClientRepo, ReorderItem, UpdateDownloadClientInput,
};
use crate::services::DownloadClientService;
use crate::shared::episode_parser::should_include_file;
use crate::shared::path::{normalize_category_slug, resolve_download_path};
use crate::shared::torrent_parser::parse_torrent;
use crate::subscriptions::repos::pt_site_repo::PtSiteRepo;

// ── Reusable torrent download helper ─────────────────────────────────────────

/// Download .torrent file bytes. Handles M-Team's genDlToken API flow:
/// if the URL contains "genDlToken", POST to get the real download URL first.
async fn download_torrent_bytes(
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

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTorrentsParams {
    pub filter: Option<String>,
    pub category: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTorrentBody {
    pub urls: Option<Vec<String>>,
    pub torrents: Option<Vec<String>>,
    pub save_path: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub paused: Option<bool>,
    pub skip_hash_check: Option<bool>,
}

#[derive(Deserialize)]
pub struct TorrentHashesBody {
    pub hashes: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTorrentsBody {
    pub hashes: Vec<String>,
    pub delete_files: Option<bool>,
}

#[derive(Deserialize)]
pub struct SetFilePriorityBody {
    pub hash: String,
    pub file_ids: Vec<u32>,
    pub priority: u8,
}

pub fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "data": data }))
}

fn ok_empty() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

// ── Client CRUD handlers ──────────────────────────────────────────────────────

pub async fn list_clients(State(ctx): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let clients = DownloadClientRepo::list(&ctx.db).await?;
    Ok(ok(clients))
}

pub async fn get_client(
    State(ctx): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let client = DownloadClientRepo::get_by_id(&ctx.db, &id)
        .await?
        .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;
    Ok(ok(client))
}

pub async fn create_client(
    State(ctx): State<Arc<AppState>>,
    Json(body): Json<CreateDownloadClientInput>,
) -> Result<impl IntoResponse, AppError> {
    let client = DownloadClientRepo::create(&ctx.db, body).await?;
    Ok(ok(client))
}

pub async fn update_client(
    State(ctx): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateDownloadClientInput>,
) -> Result<impl IntoResponse, AppError> {
    let client = DownloadClientRepo::update(&ctx.db, &id, body).await?;
    Ok(ok(client))
}

pub async fn delete_client(
    State(ctx): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    DownloadClientRepo::delete(&ctx.db, &id).await?;
    Ok(ok_empty())
}

pub async fn set_default(
    State(ctx): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    DownloadClientRepo::set_default(&ctx.db, &id).await?;
    Ok(ok_empty())
}

pub async fn reorder(
    State(ctx): State<Arc<AppState>>,
    Json(body): Json<Vec<ReorderItem>>,
) -> Result<impl IntoResponse, AppError> {
    DownloadClientRepo::reorder(&ctx.db, body).await?;
    Ok(ok_empty())
}

// ── Connection test ───────────────────────────────────────────────────────────

pub async fn test_connection(
    State(ctx): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let status = DownloadClientService::test_connection(&ctx.db, &id, &ctx.http_client).await?;
    Ok(ok(status))
}

pub async fn get_all_status(State(ctx): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let statuses = DownloadClientService::get_all_status(&ctx.db, &ctx.http_client).await?;
    Ok(ok(statuses))
}

// ── Torrent operations ────────────────────────────────────────────────────────

pub async fn get_torrents(
    State(ctx): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<GetTorrentsParams>,
) -> Result<impl IntoResponse, AppError> {
    let torrents = DownloadClientService::get_torrents(
        &ctx.db,
        &id,
        params.filter.as_deref(),
        params.category.as_deref(),
        &ctx.http_client,
    )
    .await?;
    Ok(ok(torrents))
}

pub async fn add_torrent(
    State(ctx): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<AddTorrentBody>,
) -> Result<impl IntoResponse, AppError> {
    let options = AddTorrentOptions {
        urls: body.urls,
        torrents: body.torrents,
        save_path: body.save_path,
        category: body.category,
        tags: body.tags,
        paused: body.paused,
        skip_hash_check: body.skip_hash_check,
    };
    DownloadClientService::add_torrent(&ctx.db, &id, options, &ctx.http_client).await?;
    Ok(ok_empty())
}

pub async fn pause_torrents(
    State(ctx): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<TorrentHashesBody>,
) -> Result<impl IntoResponse, AppError> {
    DownloadClientService::pause_torrents(&ctx.db, &id, &body.hashes, &ctx.http_client).await?;
    Ok(ok_empty())
}

pub async fn resume_torrents(
    State(ctx): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<TorrentHashesBody>,
) -> Result<impl IntoResponse, AppError> {
    DownloadClientService::resume_torrents(&ctx.db, &id, &body.hashes, &ctx.http_client).await?;
    Ok(ok_empty())
}

pub async fn delete_torrents(
    State(ctx): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<DeleteTorrentsBody>,
) -> Result<impl IntoResponse, AppError> {
    DownloadClientService::delete_torrents(
        &ctx.db,
        &id,
        &body.hashes,
        body.delete_files.unwrap_or(false),
        &ctx.http_client,
    )
    .await?;
    Ok(ok_empty())
}

pub async fn get_transfer_info(
    State(ctx): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let info = DownloadClientService::get_transfer_info(&ctx.db, &id, &ctx.http_client).await?;
    Ok(ok(info))
}

pub async fn get_torrent_files(
    State(ctx): State<Arc<AppState>>,
    Path((id, hash)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let files = DownloadClientService::get_torrent_files(&ctx.db, &id, &hash, &ctx.http_client).await?;
    Ok(ok(files))
}

pub async fn set_file_priority(
    State(ctx): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<SetFilePriorityBody>,
) -> Result<impl IntoResponse, AppError> {
    DownloadClientService::set_file_priority(
        &ctx.db,
        &id,
        &body.hash,
        &body.file_ids,
        body.priority,
        &ctx.http_client,
    )
    .await?;
    Ok(ok_empty())
}

// ── Torrent file preview ──────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewTorrentBody {
    pub site_id: String,
    pub torrent_id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewFileItem {
    pub index: usize,
    pub path: String,
    pub size: u64,
    pub selected: bool,
}

pub async fn preview_torrent_files(
    State(ctx): State<Arc<AppState>>,
    Json(body): Json<PreviewTorrentBody>,
) -> Result<impl IntoResponse, AppError> {
    let site = PtSiteRepo::get_by_id(&ctx.db, &body.site_id)
        .await?
        .ok_or_else(|| AppError::not_found("站点不存在"))?;

    let site_config = tokimo_pt_search::get_site_config(&site.site_id);
    let is_api = site_config
        .as_ref()
        .is_some_and(|c| c.site_type == tokimo_pt_search::SiteType::Api);
    let domain = site.domain.as_str();

    let (download_url, torrent_id) = if is_api {
        (
            format!("{}/api/torrent/genDlToken", domain.trim_end_matches('/')),
            Some(body.torrent_id.clone()),
        )
    } else {
        (
            format!("{}/download.php?id={}", domain.trim_end_matches('/'), body.torrent_id),
            None,
        )
    };

    let bytes = download_torrent_bytes(
        &ctx.http_client,
        &download_url,
        site.api_key.as_deref(),
        torrent_id.as_deref(),
    )
    .await?;

    let meta = parse_torrent(&bytes).map_err(AppError::bad_request)?;

    let files: Vec<PreviewFileItem> = meta
        .files
        .into_iter()
        .map(|f| PreviewFileItem {
            index: f.index,
            path: f.path,
            size: f.size,
            selected: true,
        })
        .collect();

    Ok(ok(files))
}

// ── Resolve download path ────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvePathBody {
    pub client_id: String,
    pub category: Option<String>,
}

pub async fn resolve_save_path(
    State(ctx): State<Arc<AppState>>,
    Json(body): Json<ResolvePathBody>,
) -> Result<impl IntoResponse, AppError> {
    let client = DownloadClientRepo::get_by_id(&ctx.db, &body.client_id)
        .await?
        .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;

    let paths: Vec<(String, String, String)> = client
        .download_paths
        .iter()
        .map(|p| (p.r#type.clone(), p.path.clone(), p.description.clone()))
        .collect();

    let cat_raw = body.category.as_deref().unwrap_or("global");
    let cat = normalize_category_slug(cat_raw);
    let resolved = resolve_download_path(&paths, &cat);

    Ok(ok(serde_json::json!({
        "path": resolved,
        "allPaths": client.download_paths,
    })))
}

// ── Download with episode filter ──────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadWithFilterBody {
    pub client_id: String,
    pub site_id: String,
    pub torrent_id: String,
    pub save_path: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub season: Option<i32>,
    pub episodes: Option<Vec<i32>>,
}

#[allow(clippy::too_many_lines)]
pub async fn download_with_filter(
    State(ctx): State<Arc<AppState>>,
    Json(body): Json<DownloadWithFilterBody>,
) -> Result<impl IntoResponse, AppError> {
    // Look up site config
    let site = PtSiteRepo::get_by_id(&ctx.db, &body.site_id)
        .await?
        .ok_or_else(|| AppError::not_found("站点不存在"))?;

    let site_config = tokimo_pt_search::get_site_config(&site.site_id);
    let is_api = site_config
        .as_ref()
        .is_some_and(|c| c.site_type == tokimo_pt_search::SiteType::Api);
    let domain = site.domain.as_str();

    let (download_url, torrent_id) = if is_api {
        (
            format!("{}/api/torrent/genDlToken", domain.trim_end_matches('/')),
            Some(body.torrent_id.clone()),
        )
    } else {
        (
            format!("{}/download.php?id={}", domain.trim_end_matches('/'), body.torrent_id),
            None,
        )
    };

    // 1. Download .torrent file (handles genDlToken for API sites)
    let bytes = download_torrent_bytes(
        &ctx.http_client,
        &download_url,
        site.api_key.as_deref(),
        torrent_id.as_deref(),
    )
    .await?;
    let meta = parse_torrent(&bytes).map_err(AppError::bad_request)?;

    // 2. Determine which files to exclude based on episode filter
    let filter_season = body.season;
    let filter_episodes = body.episodes.as_deref().unwrap_or(&[]);
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
    let save_path = if body.save_path.is_some() {
        body.save_path.clone()
    } else {
        let client = DownloadClientRepo::get_by_id(&ctx.db, &body.client_id)
            .await?
            .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;
        let paths: Vec<(String, String, String)> = client
            .download_paths
            .iter()
            .map(|p| (p.r#type.clone(), p.path.clone(), p.description.clone()))
            .collect();
        let cat_raw = body.category.as_deref().unwrap_or("global");
        resolve_download_path(&paths, &normalize_category_slug(cat_raw))
    };

    // 4. Add torrent to download client (paused if we need to filter files)
    let need_filter = !excluded_indices.is_empty();
    let torrent_bytes = base64::Engine::encode(&base64::engine::general_purpose::STANDARD_NO_PAD, &bytes);

    let options = AddTorrentOptions {
        urls: None,
        torrents: Some(vec![torrent_bytes]),
        save_path: save_path.clone(),
        category: body.category.clone(),
        tags: body.tags.clone(),
        paused: if need_filter { Some(true) } else { Some(false) },
        skip_hash_check: None,
    };

    DownloadClientService::add_torrent(&ctx.db, &body.client_id, options, &ctx.http_client).await?;

    // 4. If we need to filter, find the torrent by name and set file priorities
    if need_filter {
        // Wait briefly for qBittorrent to register the torrent
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Find the torrent by name
        let torrents =
            DownloadClientService::get_torrents(&ctx.db, &body.client_id, None, None, &ctx.http_client).await?;

        let target = torrents.iter().find(|t| t.name == meta.name);
        if let Some(torrent) = target {
            // Set excluded files to priority 0 (don't download)
            let hash = &torrent.hash;
            DownloadClientService::set_file_priority(
                &ctx.db,
                &body.client_id,
                hash,
                &excluded_indices,
                0, // don't download
                &ctx.http_client,
            )
            .await?;

            // Resume the torrent
            DownloadClientService::resume_torrents(
                &ctx.db,
                &body.client_id,
                std::slice::from_ref(hash),
                &ctx.http_client,
            )
            .await?;

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
            // Can't filter, just resume all
        }
    }

    Ok(ok(serde_json::json!({
        "totalFiles": meta.files.len(),
        "excludedFiles": excluded_indices.len(),
        "torrentName": meta.name,
    })))
}
