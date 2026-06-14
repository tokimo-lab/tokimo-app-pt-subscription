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

pub async fn get_client(State(ctx): State<Arc<AppState>>, Path(id): Path<String>) -> Result<impl IntoResponse, AppError> {
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
