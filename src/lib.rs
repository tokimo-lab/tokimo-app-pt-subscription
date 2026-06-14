//! Library facade — exposes modules for ts-rs type generation and testing.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tokimo_package_storage::StorageProvider;

/// Compile-time embedded app manifest, used by the db module to read the schema name.
pub(crate) const MANIFEST: &str = include_str!("../tokimo-app.toml");

pub mod db;
pub mod handlers;
pub mod services;
pub mod shared;
pub mod subscriptions;

/// Re-export subscriptions under `apps::subscriptions` for code copied from video app.
pub mod apps {
    pub mod subscriptions {
        pub use crate::subscriptions::*;
    }
}

// ── AppError (enum, matches video app pattern) ───────────────────────────────

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    BadRequest(String),
    Forbidden(String),
    Conflict(String),
    Internal(String),
    Gone(String),
    Database(sea_orm::DbErr),
}

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
            Self::Unauthorized(msg) => write!(f, "unauthorized: {msg}"),
            Self::BadRequest(msg) => write!(f, "bad request: {msg}"),
            Self::Forbidden(msg) => write!(f, "forbidden: {msg}"),
            Self::Conflict(msg) => write!(f, "conflict: {msg}"),
            Self::Internal(msg) => write!(f, "internal: {msg}"),
            Self::Gone(msg) => write!(f, "gone: {msg}"),
            Self::Database(err) => write!(f, "database: {err}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        Self::Database(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::Internal(format!("JSON error: {err}"))
    }
}

pub trait OptionExt<T> {
    fn not_found(self, msg: impl Into<String>) -> Result<T, AppError>;
    fn bad_request(self, msg: impl Into<String>) -> Result<T, AppError>;
    fn unauthorized(self, msg: impl Into<String>) -> Result<T, AppError>;
    fn internal(self, msg: impl Into<String>) -> Result<T, AppError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn not_found(self, msg: impl Into<String>) -> Result<T, AppError> {
        self.ok_or_else(|| AppError::NotFound(msg.into()))
    }
    fn bad_request(self, msg: impl Into<String>) -> Result<T, AppError> {
        self.ok_or_else(|| AppError::BadRequest(msg.into()))
    }
    fn unauthorized(self, msg: impl Into<String>) -> Result<T, AppError> {
        self.ok_or_else(|| AppError::Unauthorized(msg.into()))
    }
    fn internal(self, msg: impl Into<String>) -> Result<T, AppError> {
        self.ok_or_else(|| AppError::Internal(msg.into()))
    }
}

#[derive(Serialize)]
struct ErrorBody {
    success: bool,
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Self::Gone(msg) => (StatusCode::GONE, msg.clone()),
            Self::Database(err) => {
                tracing::error!("database error: {err}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal database error".to_string())
            }
        };
        (
            status,
            Json(ErrorBody {
                success: false,
                error: message,
            }),
        )
            .into_response()
    }
}

// ── AppState ─────────────────────────────────────────────────────────────────

pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    #[allow(dead_code)]
    pub client: Arc<OnceLock<Arc<tokimo_bus_client::BusClient>>>,
    pub http_client: reqwest::Client,
    pub storage: Arc<OnceLock<Arc<dyn StorageProvider>>>,
    pub active_subscription_runs: Arc<RwLock<HashMap<String, String>>>,
}

impl AppState {
    pub fn storage(&self) -> &Arc<dyn StorageProvider> {
        self.storage
            .get()
            .expect("storage not initialized — bus client must be connected first")
    }
}
