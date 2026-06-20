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
use crate::services::torrent_download::{DownloadTorrentParams, download_torrent_bytes, download_torrent_to_client};
use crate::shared::path::{normalize_category_slug, resolve_download_path};
use crate::shared::torrent_parser::parse_torrent;
use crate::subscriptions::repos::pt_site_repo::PtSiteRepo;

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

pub async fn download_with_filter(
    State(ctx): State<Arc<AppState>>,
    Json(body): Json<DownloadWithFilterBody>,
) -> Result<impl IntoResponse, AppError> {
    let outcome = download_torrent_to_client(
        &ctx.db,
        &ctx.http_client,
        DownloadTorrentParams {
            site_id: body.site_id,
            torrent_id: body.torrent_id,
            client_id: body.client_id,
            category: body.category,
            save_path: body.save_path,
            season: body.season,
            episodes: body.episodes,
            tags: body.tags,
            paused: None,
        },
    )
    .await?;

    Ok(ok(serde_json::json!({
        "totalFiles": outcome.total_files,
        "excludedFiles": outcome.excluded_files,
        "torrentName": outcome.torrent_name,
    })))
}
