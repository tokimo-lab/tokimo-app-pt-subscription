use crate::db::{ApiDateTimeExt, OptionalApiDateTimeExt};
use chrono::Utc;
use sea_orm::{JsonValue as Json, *};
use serde::Serialize;
use uuid::Uuid;

use crate::AppError;
use crate::db::entities::subscriptions;

// ── DTO ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionDto {
    pub id: String,
    pub media_type: String,
    pub tmdb_id: Option<i64>,
    pub title: String,
    pub year: Option<String>,
    pub poster_path: Option<String>,
    pub season: Option<i32>,
    pub episodes: Option<Vec<i32>>,
    pub category: Option<String>,
    pub sources: Option<Vec<String>>,
    pub resolutions: Option<Vec<String>>,
    pub codecs: Option<Vec<String>>,
    pub release_groups: Option<Vec<String>>,
    pub min_size: f64,
    pub max_size: f64,
    pub min_seeders: f64,
    pub max_seeders: f64,
    pub include_keywords: Option<String>,
    pub exclude_keywords: Option<String>,
    pub free_only: bool,
    pub exclude_hr: bool,
    pub max_downloads_per_run: i32,
    pub status: String,
    pub interval_minutes: i32,
    pub site_ids: Option<Vec<String>>,
    pub download_client_id: Option<String>,
    pub last_checked_at: Option<String>,
    pub next_check_at: Option<String>,
    pub created_by: Option<String>,
    pub created_by_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeProgress {
    pub downloaded_episodes: Vec<i32>,
    pub total_episodes: Option<i32>,
}

// ── Input types ─────────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubscriptionInput {
    pub media_type: String,
    pub tmdb_id: Option<i64>,
    pub title: String,
    pub year: Option<String>,
    pub poster_path: Option<String>,
    pub season: Option<i32>,
    pub episodes: Option<Vec<i32>>,
    pub category: Option<String>,
    pub sources: Option<Vec<String>>,
    pub resolutions: Option<Vec<String>>,
    pub codecs: Option<Vec<String>>,
    pub release_groups: Option<Vec<String>>,
    #[serde(default)]
    pub min_size: Option<f64>,
    #[serde(default)]
    pub max_size: Option<f64>,
    #[serde(default)]
    pub min_seeders: Option<f64>,
    #[serde(default)]
    pub max_seeders: Option<f64>,
    pub include_keywords: Option<String>,
    pub exclude_keywords: Option<String>,
    #[serde(default)]
    pub free_only: Option<bool>,
    #[serde(default)]
    pub exclude_hr: Option<bool>,
    pub max_downloads_per_run: Option<i32>,
    pub interval_minutes: Option<i32>,
    pub site_ids: Option<Vec<String>>,
    pub download_client_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubscriptionInput {
    #[serde(default)]
    pub id: String,
    pub season: Option<Option<i32>>,
    pub episodes: Option<Option<Vec<i32>>>,
    pub category: Option<Option<String>>,
    pub sources: Option<Option<Vec<String>>>,
    pub resolutions: Option<Option<Vec<String>>>,
    pub codecs: Option<Option<Vec<String>>>,
    pub release_groups: Option<Option<Vec<String>>>,
    pub min_size: Option<f64>,
    pub max_size: Option<f64>,
    pub min_seeders: Option<f64>,
    pub max_seeders: Option<f64>,
    pub include_keywords: Option<Option<String>>,
    pub exclude_keywords: Option<Option<String>>,
    pub free_only: Option<bool>,
    pub exclude_hr: Option<bool>,
    pub status: Option<String>,
    pub interval_minutes: Option<i32>,
    pub max_downloads_per_run: Option<i32>,
    pub site_ids: Option<Option<Vec<String>>>,
    pub download_client_id: Option<Option<String>>,
}

// ── Conversion helpers ──────────────────────────────────────────────────────

fn json_to_string_vec(val: &Option<Json>) -> Option<Vec<String>> {
    val.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok())
}

fn json_to_int_vec(val: &Option<Json>) -> Option<Vec<i32>> {
    val.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok())
}

fn parse_num(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or(0.0)
}

fn to_dto(model: &subscriptions::Model, created_by_name: Option<String>) -> SubscriptionDto {
    SubscriptionDto {
        id: model.id.to_string(),
        media_type: model.media_type.clone(),
        tmdb_id: model.tmdb_id.as_ref().and_then(|s| s.parse::<i64>().ok()),
        title: model.title.clone(),
        year: model.year.clone(),
        poster_path: model.poster_path.clone(),
        season: model.season.as_ref().and_then(|s| s.parse::<i32>().ok()),
        episodes: json_to_int_vec(&model.episodes),
        category: model.category.clone(),
        sources: json_to_string_vec(&model.sources),
        resolutions: json_to_string_vec(&model.resolutions),
        codecs: json_to_string_vec(&model.codecs),
        release_groups: json_to_string_vec(&model.release_groups),
        min_size: parse_num(&model.min_size),
        max_size: parse_num(&model.max_size),
        min_seeders: parse_num(&model.min_seeders),
        max_seeders: parse_num(&model.max_seeders),
        include_keywords: model.include_keywords.clone(),
        exclude_keywords: model.exclude_keywords.clone(),
        free_only: model.free_only,
        exclude_hr: model.exclude_hr,
        max_downloads_per_run: model.max_downloads_per_run,
        status: model.status.clone(),
        interval_minutes: model.interval_minutes.parse::<i32>().unwrap_or(30),
        site_ids: json_to_string_vec(&model.site_ids),
        download_client_id: model.download_client_id.map(|id| id.to_string()),
        last_checked_at: model.last_checked_at.to_api_datetime(),
        next_check_at: model.next_check_at.to_api_datetime(),
        created_by: model.created_by.map(|id| id.to_string()),
        created_by_name,
        created_at: model.created_at.to_api_datetime_or_default(),
        updated_at: model.updated_at.to_api_datetime_or_default(),
    }
}

// ── Repo ────────────────────────────────────────────────────────────────────

pub struct SubscriptionRepo;

impl SubscriptionRepo {
    pub async fn list<C: ConnectionTrait>(db: &C, user_id: &str) -> Result<Vec<SubscriptionDto>, AppError> {
        let uid: Uuid = user_id
            .parse()
            .map_err(|_| AppError::BadRequest("invalid user id".into()))?;

        let rows = subscriptions::Entity::find()
            .filter(subscriptions::Column::CreatedBy.eq(uid))
            .order_by_desc(subscriptions::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(rows.iter().map(|sub| to_dto(sub, None)).collect())
    }

    pub async fn get_by_id<C: ConnectionTrait>(db: &C, id: &str) -> Result<Option<SubscriptionDto>, AppError> {
        let uid: Uuid = id
            .parse()
            .map_err(|_| AppError::BadRequest("invalid subscription id".into()))?;

        let row = subscriptions::Entity::find_by_id(uid).one(db).await?;
        Ok(row.map(|sub| to_dto(&sub, None)))
    }

    pub async fn get_raw<C: ConnectionTrait>(db: &C, id: &str) -> Result<Option<subscriptions::Model>, AppError> {
        let uid: Uuid = id
            .parse()
            .map_err(|_| AppError::BadRequest("invalid subscription id".into()))?;
        Ok(subscriptions::Entity::find_by_id(uid).one(db).await?)
    }

    pub async fn create<C: ConnectionTrait>(
        db: &C,
        input: CreateSubscriptionInput,
        user_id: &str,
    ) -> Result<SubscriptionDto, AppError> {
        let user_uuid: Uuid = user_id
            .parse()
            .map_err(|_| AppError::BadRequest("invalid user id".into()))?;
        let now = Utc::now().fixed_offset();

        let active = subscriptions::ActiveModel {
            id: Set(Uuid::new_v4()),
            media_type: Set(input.media_type),
            tmdb_id: Set(input.tmdb_id.map(|n| n.to_string())),
            title: Set(input.title),
            year: Set(input.year),
            poster_path: Set(input.poster_path),
            season: Set(input.season.map(|n| n.to_string())),
            episodes: Set(input.episodes.map(|e| serde_json::to_value(e).unwrap_or_default())),
            category: Set(input.category),
            sources: Set(input.sources.map(|v| serde_json::to_value(v).unwrap_or_default())),
            resolutions: Set(input.resolutions.map(|v| serde_json::to_value(v).unwrap_or_default())),
            codecs: Set(input.codecs.map(|v| serde_json::to_value(v).unwrap_or_default())),
            release_groups: Set(input
                .release_groups
                .map(|v| serde_json::to_value(v).unwrap_or_default())),
            min_size: Set(input.min_size.unwrap_or(0.0).to_string()),
            max_size: Set(input.max_size.unwrap_or(0.0).to_string()),
            min_seeders: Set(input.min_seeders.unwrap_or(0.0).to_string()),
            max_seeders: Set(input.max_seeders.unwrap_or(0.0).to_string()),
            include_keywords: Set(input.include_keywords),
            exclude_keywords: Set(input.exclude_keywords),
            free_only: Set(input.free_only.unwrap_or(false)),
            exclude_hr: Set(input.exclude_hr.unwrap_or(false)),
            max_downloads_per_run: Set(input.max_downloads_per_run.unwrap_or(10)),
            status: Set("active".to_string()),
            interval_minutes: Set(input.interval_minutes.unwrap_or(5).to_string()),
            site_ids: Set(input.site_ids.map(|ids| serde_json::to_value(ids).unwrap_or_default())),
            download_client_id: Set(input.download_client_id.as_ref().and_then(|id| id.parse::<Uuid>().ok())),
            last_checked_at: Set(None),
            next_check_at: Set(Some(now)),
            created_by: Set(Some(user_uuid)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
        };

        let model = subscriptions::Entity::insert(active).exec_with_returning(db).await?;
        Ok(to_dto(&model, None))
    }

    pub async fn update<C: ConnectionTrait>(
        db: &C,
        id: &str,
        input: UpdateSubscriptionInput,
    ) -> Result<Option<SubscriptionDto>, AppError> {
        let uid: Uuid = id
            .parse()
            .map_err(|_| AppError::BadRequest("invalid subscription id".into()))?;

        let Some(existing) = subscriptions::Entity::find_by_id(uid).one(db).await? else {
            return Ok(None);
        };

        let mut active: subscriptions::ActiveModel = existing.into();
        let now = Utc::now().fixed_offset();
        active.updated_at = Set(Some(now));

        if let Some(season) = input.season {
            active.season = Set(season.map(|n| n.to_string()));
        }
        if let Some(episodes) = input.episodes {
            active.episodes = Set(episodes.map(|e| serde_json::to_value(e).unwrap_or_default()));
        }
        if let Some(category) = input.category {
            active.category = Set(category);
        }
        if let Some(sources) = input.sources {
            active.sources = Set(sources.map(|v| serde_json::to_value(v).unwrap_or_default()));
        }
        if let Some(resolutions) = input.resolutions {
            active.resolutions = Set(resolutions.map(|v| serde_json::to_value(v).unwrap_or_default()));
        }
        if let Some(codecs) = input.codecs {
            active.codecs = Set(codecs.map(|v| serde_json::to_value(v).unwrap_or_default()));
        }
        if let Some(release_groups) = input.release_groups {
            active.release_groups = Set(release_groups.map(|v| serde_json::to_value(v).unwrap_or_default()));
        }
        if let Some(min_size) = input.min_size {
            active.min_size = Set(min_size.to_string());
        }
        if let Some(max_size) = input.max_size {
            active.max_size = Set(max_size.to_string());
        }
        if let Some(min_seeders) = input.min_seeders {
            active.min_seeders = Set(min_seeders.to_string());
        }
        if let Some(max_seeders) = input.max_seeders {
            active.max_seeders = Set(max_seeders.to_string());
        }
        if let Some(include_keywords) = input.include_keywords {
            active.include_keywords = Set(include_keywords);
        }
        if let Some(exclude_keywords) = input.exclude_keywords {
            active.exclude_keywords = Set(exclude_keywords);
        }
        if let Some(free_only) = input.free_only {
            active.free_only = Set(free_only);
        }
        if let Some(exclude_hr) = input.exclude_hr {
            active.exclude_hr = Set(exclude_hr);
        }
        if let Some(status) = input.status {
            active.status = Set(status);
        }
        if let Some(interval_minutes) = input.interval_minutes {
            active.interval_minutes = Set(interval_minutes.to_string());
        }
        if let Some(max_downloads) = input.max_downloads_per_run {
            active.max_downloads_per_run = Set(max_downloads);
        }
        if let Some(site_ids) = input.site_ids {
            active.site_ids = Set(site_ids.map(|ids| serde_json::to_value(ids).unwrap_or_default()));
        }
        if let Some(download_client_id) = input.download_client_id {
            active.download_client_id = Set(download_client_id.and_then(|id| id.parse::<Uuid>().ok()));
        }

        let updated = active.update(db).await?;
        Ok(Some(to_dto(&updated, None)))
    }

    pub async fn delete<C: ConnectionTrait>(db: &C, id: &str) -> Result<bool, AppError> {
        let uid: Uuid = id
            .parse()
            .map_err(|_| AppError::BadRequest("invalid subscription id".into()))?;
        let result = subscriptions::Entity::delete_by_id(uid).exec(db).await?;
        Ok(result.rows_affected > 0)
    }

    pub async fn update_timestamps<C: ConnectionTrait>(
        db: &C,
        id: &str,
        interval_minutes: i32,
    ) -> Result<(), AppError> {
        let uid: Uuid = id
            .parse()
            .map_err(|_| AppError::BadRequest("invalid subscription id".into()))?;

        let Some(existing) = subscriptions::Entity::find_by_id(uid).one(db).await? else {
            return Err(AppError::NotFound("subscription not found".into()));
        };

        let now = Utc::now().fixed_offset();
        let next = now + chrono::Duration::minutes(i64::from(interval_minutes));

        let mut active: subscriptions::ActiveModel = existing.into();
        active.last_checked_at = Set(Some(now));
        active.next_check_at = Set(Some(next));
        active.updated_at = Set(Some(now));
        active.update(db).await?;
        Ok(())
    }

    pub async fn get_episode_progress<C: ConnectionTrait>(
        db: &C,
        subscription_id: &str,
    ) -> Result<EpisodeProgress, AppError> {
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            r"SELECT app_metadata FROM download_records
               WHERE app_metadata->>'subscriptionId' = $1
                 AND status != 'failed'",
            [subscription_id.into()],
        );

        #[derive(Debug, sea_orm::FromQueryResult)]
        struct Row {
            app_metadata: Option<serde_json::Value>,
        }

        let rows = Row::find_by_statement(stmt).all(db).await.map_err(|e| {
            tracing::error!("get_episode_progress query failed: {e}");
            AppError::Database(e)
        })?;

        let mut downloaded = std::collections::BTreeSet::new();
        for row in &rows {
            let app_meta = row.app_metadata.as_ref().and_then(serde_json::Value::as_object);
            let episodes = app_meta.and_then(|obj| obj.get("episodes")).cloned();
            let episode = app_meta
                .and_then(|obj| obj.get("episode"))
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned);

            if let Some(eps) = episodes
                .as_ref()
                .and_then(|v| serde_json::from_value::<Vec<i32>>(v.clone()).ok())
            {
                for ep in eps {
                    downloaded.insert(ep);
                }
            } else if let Some(ep_str) = episode
                && let Ok(ep) = ep_str.parse::<i32>()
            {
                downloaded.insert(ep);
            }
        }

        Ok(EpisodeProgress {
            downloaded_episodes: downloaded.into_iter().collect(),
            total_episodes: None,
        })
    }
}
