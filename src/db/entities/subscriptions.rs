//! `SeaORM` Entity — pt_subscription schema.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "pt_subscription", table_name = "subscriptions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_type = "Text")]
    pub media_type: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub tmdb_id: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub title: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub year: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub poster_path: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub season: Option<String>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub episodes: Option<Json>,
    #[sea_orm(column_type = "Text", nullable)]
    pub category: Option<String>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub sources: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub resolutions: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub codecs: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub release_groups: Option<Json>,
    #[sea_orm(column_type = "Text")]
    pub min_size: String,
    #[sea_orm(column_type = "Text")]
    pub max_size: String,
    #[sea_orm(column_type = "Text")]
    pub min_seeders: String,
    #[sea_orm(column_type = "Text")]
    pub max_seeders: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub include_keywords: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub exclude_keywords: Option<String>,
    pub free_only: bool,
    pub exclude_hr: bool,
    pub max_downloads_per_run: i32,
    #[sea_orm(column_type = "Text")]
    pub status: String,
    #[sea_orm(column_type = "Text")]
    pub interval_minutes: String,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub site_ids: Option<Json>,
    pub download_client_id: Option<Uuid>,
    pub last_checked_at: Option<DateTimeWithTimeZone>,
    pub next_check_at: Option<DateTimeWithTimeZone>,
    pub created_by: Option<Uuid>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::download_clients::Entity",
        from = "Column::DownloadClientId",
        to = "super::download_clients::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    DownloadClients,
}

impl Related<super::download_clients::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DownloadClients.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
