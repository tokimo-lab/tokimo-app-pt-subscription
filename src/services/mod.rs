pub mod torrent_download;

use sea_orm::DatabaseConnection;
use serde::Serialize;
use tokimo_package_client_api::downloaders::{
    aria2::{Aria2Client, Aria2Config},
    deluge::{DelugeClient, DelugeConfig},
    pan115::{Pan115Client, Pan115Config},
    qbittorrent::{QBittorrentClient, QBittorrentConfig},
    rtorrent::{RTorrentClient, RTorrentConfig},
    traits::{AddTorrentOptions, DownloadClient, TorrentInfo, TorrentState, TransferInfo},
    transmission::{TransmissionClient, TransmissionConfig},
    xunlei::{XunleiClient, XunleiConfig},
};
use ts_rs::TS;

use crate::AppError;
use crate::db::{
    entities::download_clients,
    repos::download_client_repo::{DownloadClientDto, DownloadClientRepo},
};

// ── Client factory ────────────────────────────────────────────────────────────

pub(crate) enum AnyDownloadClient {
    QBittorrent(QBittorrentClient),
    Transmission(TransmissionClient),
    Aria2(Aria2Client),
    Deluge(DelugeClient),
    RTorrent(RTorrentClient),
    Xunlei(XunleiClient),
    Pan115(Pan115Client),
}

pub(crate) fn make_client(
    model: &download_clients::Model,
    http_client: &reqwest::Client,
) -> Result<AnyDownloadClient, AppError> {
    let url = model.url.clone();
    let username = model.username.clone().unwrap_or_default();
    let password = model.password.clone().unwrap_or_default();

    match model.r#type.as_str() {
        "qbittorrent" => Ok(AnyDownloadClient::QBittorrent(QBittorrentClient::new(
            QBittorrentConfig {
                url,
                username,
                password,
            },
        ))),
        "transmission" => Ok(AnyDownloadClient::Transmission(TransmissionClient::new(
            TransmissionConfig {
                url,
                username: model.username.clone(),
                password: model.password.clone(),
                http_client: http_client.clone(),
            },
        ))),
        "aria2" => Ok(AnyDownloadClient::Aria2(Aria2Client::new(Aria2Config {
            url,
            secret: model.password.clone(),
            http_client: http_client.clone(),
        }))),
        "deluge" => Ok(AnyDownloadClient::Deluge(DelugeClient::new(DelugeConfig {
            url,
            password,
        }))),
        "rtorrent" => Ok(AnyDownloadClient::RTorrent(RTorrentClient::new(RTorrentConfig {
            url,
            username: model.username.clone(),
            password: model.password.clone(),
            http_client: http_client.clone(),
        }))),
        "xunlei" => Ok(AnyDownloadClient::Xunlei(XunleiClient::new(XunleiConfig {
            url,
            username,
            password,
            http_client: http_client.clone(),
        }))),
        "pan115" => Ok(AnyDownloadClient::Pan115(Pan115Client::new(Pan115Config {
            url: Some(url),
            cookies: password,
            http_client: http_client.clone(),
        }))),
        t => Err(AppError::bad_request(format!("不支持的下载客户端类型: {t}"))),
    }
}

// ── Dispatch macro ────────────────────────────────────────────────────────────

macro_rules! dispatch {
    ($client:expr, $method:ident ( $($arg:expr),* )) => {
        match &$client {
            AnyDownloadClient::QBittorrent(c) => { eprintln!("[dispatch] -> QBittorrent"); c.$method($($arg),*).await },
            AnyDownloadClient::Transmission(c) => { eprintln!("[dispatch] -> Transmission"); c.$method($($arg),*).await },
            AnyDownloadClient::Aria2(c) => { eprintln!("[dispatch] -> Aria2"); c.$method($($arg),*).await },
            AnyDownloadClient::Deluge(c) => { eprintln!("[dispatch] -> Deluge"); c.$method($($arg),*).await },
            AnyDownloadClient::RTorrent(c) => { eprintln!("[dispatch] -> RTorrent"); c.$method($($arg),*).await },
            AnyDownloadClient::Xunlei(c) => { eprintln!("[dispatch] -> Xunlei"); c.$method($($arg),*).await },
            AnyDownloadClient::Pan115(c) => { eprintln!("[dispatch] -> Pan115"); c.$method($($arg),*).await },
        }
    };
}

// ── Response DTOs ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "ClientStatusDto")]
pub struct ClientStatusDto {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub is_connected: bool,
    pub version: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TorrentInfoDto")]
pub struct TorrentInfoDto {
    pub hash: String,
    pub name: String,
    #[ts(type = "number")]
    pub size: u64,
    pub progress: f64,
    #[ts(type = "number")]
    pub download_speed: u64,
    #[ts(type = "number")]
    pub upload_speed: u64,
    #[ts(type = "number")]
    pub downloaded: u64,
    #[ts(type = "number")]
    pub uploaded: u64,
    pub ratio: f64,
    pub state: String,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub save_path: String,
    #[ts(type = "number")]
    pub added_on: i64,
    #[ts(type = "number | null")]
    pub completed_on: Option<i64>,
    #[ts(type = "number")]
    pub seeding_time: u64,
    #[ts(type = "number | null")]
    pub eta: Option<i64>,
    #[ts(type = "number | null")]
    pub num_seeds: Option<u32>,
    #[ts(type = "number | null")]
    pub num_leeches: Option<u32>,
    pub tracker: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TransferInfoDto")]
pub struct TransferInfoDto {
    #[ts(type = "number")]
    pub dl_speed: u64,
    #[ts(type = "number")]
    pub up_speed: u64,
    #[ts(type = "number")]
    pub free_space: u64,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TorrentFileDto")]
pub struct TorrentFileDto {
    pub index: u32,
    pub name: String,
    #[ts(type = "number")]
    pub size: u64,
    pub progress: f64,
    pub priority: i32,
}

// ── Converters ────────────────────────────────────────────────────────────────

fn map_torrent_state(state: &TorrentState) -> &'static str {
    match state {
        TorrentState::Downloading => "downloading",
        TorrentState::Uploading => "uploading",
        TorrentState::Seeding => "seeding",
        TorrentState::PausedDl => "pausedDL",
        TorrentState::PausedUp => "pausedUP",
        TorrentState::QueuedDl => "queuedDL",
        TorrentState::QueuedUp => "queuedUP",
        TorrentState::CheckingDl => "checkingDL",
        TorrentState::CheckingUp => "checkingUP",
        TorrentState::StalledDl => "stalledDL",
        TorrentState::StalledUp => "stalledUP",
        TorrentState::Error => "error",
        TorrentState::MissingFiles => "missingFiles",
        TorrentState::Unknown => "unknown",
    }
}

fn to_torrent_dto(info: TorrentInfo) -> TorrentInfoDto {
    TorrentInfoDto {
        state: map_torrent_state(&info.state).to_string(),
        hash: info.hash,
        name: info.name,
        size: info.size,
        progress: info.progress,
        download_speed: info.dl_speed,
        upload_speed: info.up_speed,
        downloaded: info.downloaded,
        uploaded: info.uploaded,
        ratio: info.ratio,
        category: if info.category.is_empty() {
            None
        } else {
            Some(info.category)
        },
        tags: if info.tags.is_empty() { None } else { Some(info.tags) },
        save_path: info.save_path,
        added_on: info.added_on,
        completed_on: info.completion_on,
        seeding_time: info.seeding_time.unwrap_or(0),
        eta: info.eta,
        num_seeds: info.seeds,
        num_leeches: info.peers,
        tracker: info.tracker,
    }
}

fn to_transfer_dto(info: TransferInfo) -> TransferInfoDto {
    TransferInfoDto {
        dl_speed: info.dl_speed,
        up_speed: info.up_speed,
        free_space: info.free_space,
    }
}

fn to_torrent_file_dto(f: tokimo_package_client_api::downloaders::traits::TorrentFile) -> TorrentFileDto {
    TorrentFileDto {
        index: f.index,
        name: f.name,
        size: f.size,
        progress: f.progress,
        priority: f.priority,
    }
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct DownloadClientService;

impl DownloadClientService {
    pub async fn test_connection(
        db: &DatabaseConnection,
        id: &str,
        http_client: &reqwest::Client,
    ) -> Result<ClientStatusDto, AppError> {
        let model = DownloadClientRepo::get_model(db, id)
            .await?
            .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;
        let name = model.name.clone();
        let client_type = model.r#type.clone();
        let client = make_client(&model, http_client)?;
        let status = dispatch!(client, test_connection());
        match status {
            Ok(s) => Ok(ClientStatusDto {
                id: id.to_string(),
                name,
                r#type: client_type,
                is_connected: s.connected,
                version: s.version,
                error_message: s.error,
            }),
            Err(e) => Ok(ClientStatusDto {
                id: id.to_string(),
                name,
                r#type: client_type,
                is_connected: false,
                version: None,
                error_message: Some(e.to_string()),
            }),
        }
    }

    pub async fn get_torrents(
        db: &DatabaseConnection,
        id: &str,
        filter: Option<&str>,
        category: Option<&str>,
        http_client: &reqwest::Client,
    ) -> Result<Vec<TorrentInfoDto>, AppError> {
        let model = DownloadClientRepo::get_model(db, id)
            .await?
            .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;
        let client = make_client(&model, http_client)?;
        let torrents =
            dispatch!(client, get_torrents(filter, category)).map_err(|e| AppError::bad_request(e.to_string()))?;
        Ok(torrents.into_iter().map(to_torrent_dto).collect())
    }

    pub async fn add_torrent(
        db: &DatabaseConnection,
        id: &str,
        options: AddTorrentOptions,
        http_client: &reqwest::Client,
    ) -> Result<(), AppError> {
        let model = DownloadClientRepo::get_model(db, id)
            .await?
            .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;
        let client = make_client(&model, http_client)?;
        dispatch!(client, add_torrent(options)).map_err(|e| AppError::bad_request(e.to_string()))
    }

    pub async fn pause_torrents(
        db: &DatabaseConnection,
        id: &str,
        hashes: &[String],
        http_client: &reqwest::Client,
    ) -> Result<(), AppError> {
        let model = DownloadClientRepo::get_model(db, id)
            .await?
            .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;
        let client = make_client(&model, http_client)?;
        let refs: Vec<&str> = hashes.iter().map(String::as_str).collect();
        dispatch!(client, pause_torrents(&refs)).map_err(|e| AppError::bad_request(e.to_string()))
    }

    pub async fn resume_torrents(
        db: &DatabaseConnection,
        id: &str,
        hashes: &[String],
        http_client: &reqwest::Client,
    ) -> Result<(), AppError> {
        let model = DownloadClientRepo::get_model(db, id)
            .await?
            .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;
        let client = make_client(&model, http_client)?;
        let refs: Vec<&str> = hashes.iter().map(String::as_str).collect();
        dispatch!(client, resume_torrents(&refs)).map_err(|e| AppError::bad_request(e.to_string()))
    }

    pub async fn delete_torrents(
        db: &DatabaseConnection,
        id: &str,
        hashes: &[String],
        delete_files: bool,
        http_client: &reqwest::Client,
    ) -> Result<(), AppError> {
        let model = DownloadClientRepo::get_model(db, id)
            .await?
            .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;
        let client = make_client(&model, http_client)?;
        let refs: Vec<&str> = hashes.iter().map(String::as_str).collect();
        dispatch!(client, delete_torrents(&refs, delete_files)).map_err(|e| AppError::bad_request(e.to_string()))
    }

    pub async fn get_transfer_info(
        db: &DatabaseConnection,
        id: &str,
        http_client: &reqwest::Client,
    ) -> Result<TransferInfoDto, AppError> {
        let model = DownloadClientRepo::get_model(db, id)
            .await?
            .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;
        let client = make_client(&model, http_client)?;
        let info = dispatch!(client, get_transfer_info()).map_err(|e| AppError::bad_request(e.to_string()))?;
        Ok(to_transfer_dto(info))
    }

    pub async fn get_torrent_files(
        db: &DatabaseConnection,
        id: &str,
        hash: &str,
        http_client: &reqwest::Client,
    ) -> Result<Vec<TorrentFileDto>, AppError> {
        let model = DownloadClientRepo::get_model(db, id)
            .await?
            .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;
        let client = make_client(&model, http_client)?;
        let files = dispatch!(client, get_torrent_files(hash)).map_err(|e| AppError::bad_request(e.to_string()))?;
        Ok(files.into_iter().map(to_torrent_file_dto).collect())
    }

    pub async fn set_file_priority(
        db: &DatabaseConnection,
        id: &str,
        hash: &str,
        file_ids: &[u32],
        priority: u8,
        http_client: &reqwest::Client,
    ) -> Result<(), AppError> {
        let model = DownloadClientRepo::get_model(db, id)
            .await?
            .ok_or_else(|| AppError::not_found("下载客户端不存在"))?;
        let client = make_client(&model, http_client)?;
        dispatch!(client, set_file_priority(hash, file_ids, priority)).map_err(|e| AppError::bad_request(e.to_string()))
    }

    pub async fn get_all_status(
        db: &DatabaseConnection,
        http_client: &reqwest::Client,
    ) -> Result<Vec<ClientStatusDto>, AppError> {
        let clients = DownloadClientRepo::list(db).await?;
        let db = db.clone();
        let http_client = http_client.clone();
        let handles: Vec<_> = clients
            .into_iter()
            .map(|client| {
                let db = db.clone();
                let http_client = http_client.clone();
                tokio::spawn(async move { Self::test_connection(&db, &client.id, &http_client).await })
            })
            .collect();
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(status)) => results.push(status),
                Ok(Err(e)) => {
                    tracing::warn!("test_connection failed: {e}");
                }
                Err(e) => {
                    tracing::warn!("task join failed: {e}");
                }
            }
        }
        Ok(results)
    }

    #[allow(dead_code)]
    pub async fn get_default(db: &DatabaseConnection) -> Result<Option<DownloadClientDto>, AppError> {
        DownloadClientRepo::get_default(db).await
    }
}
