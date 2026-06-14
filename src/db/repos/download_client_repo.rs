use chrono::Utc;
use sea_orm::{sea_query::Expr, *};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::AppError;
use crate::db::entities::download_clients;

// ── Download path type ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DownloadPath {
    #[serde(default)]
    pub r#type: String,
    pub path: String,
    pub description: String,
}

// ── Output DTO ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "DownloadClientDto")]
pub struct DownloadClientDto {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub is_default: bool,
    pub require_auth: bool,
    pub monitor_enabled: bool,
    pub sort_order: i32,
    #[ts(type = "number")]
    pub poll_interval: i64,
    pub download_paths: Vec<DownloadPath>,
    pub created_at: String,
    pub updated_at: String,
}

fn fmt_dt(dt: Option<chrono::DateTime<chrono::FixedOffset>>) -> String {
    dt.map_or_else(|| chrono::Utc::now().to_rfc3339(), |d| d.to_rfc3339())
}

fn parse_download_paths(json: &serde_json::Value) -> Vec<DownloadPath> {
    serde_json::from_value(json.clone()).unwrap_or_default()
}

fn to_dto(m: download_clients::Model) -> DownloadClientDto {
    let poll_interval: i64 = m.poll_interval.parse().unwrap_or(5);
    DownloadClientDto {
        id: m.id.to_string(),
        name: m.name,
        r#type: m.r#type,
        url: m.url,
        username: m.username,
        password: m.password,
        is_default: m.is_default,
        require_auth: m.require_auth,
        monitor_enabled: m.monitor_enabled,
        sort_order: m.sort_order,
        poll_interval,
        download_paths: parse_download_paths(&m.download_paths),
        created_at: fmt_dt(m.created_at),
        updated_at: fmt_dt(m.updated_at),
    }
}

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDownloadClientInput {
    pub name: String,
    pub r#type: String,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub is_default: Option<bool>,
    pub require_auth: Option<bool>,
    pub monitor_enabled: Option<bool>,
    pub poll_interval: Option<i64>,
    pub download_paths: Vec<DownloadPath>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDownloadClientInput {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    #[allow(clippy::option_option)]
    pub username: Option<Option<String>>,
    #[serde(default)]
    #[allow(clippy::option_option)]
    pub password: Option<Option<String>>,
    pub is_default: Option<bool>,
    pub require_auth: Option<bool>,
    pub monitor_enabled: Option<bool>,
    pub poll_interval: Option<i64>,
    pub download_paths: Option<Vec<DownloadPath>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderItem {
    pub id: String,
    pub sort_order: i32,
}

// ── Repo ──────────────────────────────────────────────────────────────────────

pub struct DownloadClientRepo;

impl DownloadClientRepo {
    pub async fn list(db: &DatabaseConnection) -> Result<Vec<DownloadClientDto>, AppError> {
        let clients = download_clients::Entity::find()
            .order_by_asc(download_clients::Column::SortOrder)
            .all(db)
            .await?;
        Ok(clients.into_iter().map(to_dto).collect())
    }

    pub async fn get_by_id(db: &DatabaseConnection, id: &str) -> Result<Option<DownloadClientDto>, AppError> {
        let uid = Uuid::parse_str(id).map_err(|_| AppError::bad_request("Invalid ID"))?;
        let client = download_clients::Entity::find_by_id(uid).one(db).await?;
        Ok(client.map(to_dto))
    }

    #[allow(dead_code)]
    pub async fn get_default(db: &DatabaseConnection) -> Result<Option<DownloadClientDto>, AppError> {
        let client = download_clients::Entity::find()
            .filter(download_clients::Column::IsDefault.eq(true))
            .one(db)
            .await?;
        Ok(client.map(to_dto))
    }

    pub async fn get_model(db: &DatabaseConnection, id: &str) -> Result<Option<download_clients::Model>, AppError> {
        let uid = Uuid::parse_str(id).map_err(|_| AppError::bad_request("Invalid ID"))?;
        Ok(download_clients::Entity::find_by_id(uid).one(db).await?)
    }

    pub async fn create(
        db: &DatabaseConnection,
        input: CreateDownloadClientInput,
    ) -> Result<DownloadClientDto, AppError> {
        let count = download_clients::Entity::find().count(db).await?;
        let now = Utc::now().fixed_offset();
        let paths_json = serde_json::to_value(&input.download_paths)
            .map_err(|e| AppError::bad_request(format!("Invalid download_paths: {e}")))?;

        let active = download_clients::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(input.name),
            r#type: Set(input.r#type),
            url: Set(input.url),
            username: Set(input.username),
            password: Set(input.password),
            is_default: Set(input.is_default.unwrap_or(false)),
            require_auth: Set(input.require_auth.unwrap_or(true)),
            monitor_enabled: Set(input.monitor_enabled.unwrap_or(false)),
            sort_order: Set(count as i32),
            poll_interval: Set(input.poll_interval.unwrap_or(5).to_string()),
            download_paths: Set(paths_json),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
        };

        let inserted = active.insert(db).await?;
        Ok(to_dto(inserted))
    }

    pub async fn update(
        db: &DatabaseConnection,
        id: &str,
        input: UpdateDownloadClientInput,
    ) -> Result<DownloadClientDto, AppError> {
        let uid = Uuid::parse_str(id).map_err(|_| AppError::bad_request("Invalid ID"))?;
        let model = download_clients::Entity::find_by_id(uid)
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("Download client not found"))?;

        let now = Utc::now().fixed_offset();
        let mut active: download_clients::ActiveModel = model.into();

        if let Some(v) = input.name {
            active.name = Set(v);
        }
        if let Some(v) = input.r#type {
            active.r#type = Set(v);
        }
        if let Some(v) = input.url {
            active.url = Set(v);
        }
        if let Some(v) = input.username {
            active.username = Set(v);
        }
        if let Some(v) = input.password {
            active.password = Set(v);
        }
        if let Some(v) = input.is_default {
            active.is_default = Set(v);
        }
        if let Some(v) = input.require_auth {
            active.require_auth = Set(v);
        }
        if let Some(v) = input.monitor_enabled {
            active.monitor_enabled = Set(v);
        }
        if let Some(v) = input.poll_interval {
            active.poll_interval = Set(v.to_string());
        }
        if let Some(v) = input.download_paths {
            let paths_json =
                serde_json::to_value(&v).map_err(|e| AppError::bad_request(format!("Invalid download_paths: {e}")))?;
            active.download_paths = Set(paths_json);
        }
        active.updated_at = Set(Some(now));

        let updated = active.update(db).await?;
        Ok(to_dto(updated))
    }

    pub async fn delete(db: &DatabaseConnection, id: &str) -> Result<(), AppError> {
        let uid = Uuid::parse_str(id).map_err(|_| AppError::bad_request("Invalid ID"))?;
        download_clients::Entity::delete_by_id(uid).exec(db).await?;
        Ok(())
    }

    pub async fn set_default(db: &DatabaseConnection, id: &str) -> Result<(), AppError> {
        let uid = Uuid::parse_str(id).map_err(|_| AppError::bad_request("Invalid ID"))?;
        let now = Utc::now().fixed_offset();

        download_clients::Entity::update_many()
            .col_expr(download_clients::Column::IsDefault, Expr::value(false))
            .col_expr(download_clients::Column::UpdatedAt, Expr::value(now))
            .exec(db)
            .await?;

        download_clients::Entity::update_many()
            .col_expr(download_clients::Column::IsDefault, Expr::value(true))
            .col_expr(download_clients::Column::UpdatedAt, Expr::value(now))
            .filter(download_clients::Column::Id.eq(uid))
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn reorder(db: &DatabaseConnection, items: Vec<ReorderItem>) -> Result<(), AppError> {
        for item in items {
            let uid = Uuid::parse_str(&item.id).map_err(|_| AppError::bad_request("Invalid ID"))?;
            if let Some(model) = download_clients::Entity::find_by_id(uid).one(db).await? {
                let mut active: download_clients::ActiveModel = model.into();
                active.sort_order = Set(item.sort_order);
                active.update(db).await?;
            }
        }
        Ok(())
    }
}
