use chrono::{DateTime, FixedOffset};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "pt_subscription", table_name = "download_clients")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub r#type: String,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub is_default: bool,
    pub require_auth: bool,
    pub monitor_enabled: bool,
    pub sort_order: i32,
    pub poll_interval: String,
    pub download_paths: Json,
    pub created_at: Option<DateTime<FixedOffset>>,
    pub updated_at: Option<DateTime<FixedOffset>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
