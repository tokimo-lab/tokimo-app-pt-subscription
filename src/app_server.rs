//! Axum HTTP server on UDS — download tool management routes.
//!
//! Route layout (server-side `/api/apps/downloads/<rest>` proxies to `/<rest>` on this socket):
//!
//! Client CRUD:
//! - GET    /clients                         → list all clients
//! - POST   /clients                        → create client
//! - GET    /clients/{id}                    → get client
//! - PUT    /clients/{id}                    → update client
//! - DELETE /clients/{id}                    → delete client
//! - POST   /clients/{id}/toggle-enabled     → toggle enabled
//! - POST   /clients/{id}/set-default        → set as default
//! - POST   /clients/reorder                 → reorder clients
//! - GET    /clients/{id}/test-connection    → test connection
//! - GET    /clients/all-status              → all clients status
//!
//! Torrent operations:
//! - GET    /clients/{id}/torrents           → list torrents
//! - POST   /clients/{id}/torrents           → add torrent
//! - POST   /clients/{id}/torrents/pause     → pause torrents
//! - POST   /clients/{id}/torrents/resume    → resume torrents
//! - DELETE /clients/{id}/torrents           → delete torrents
//! - GET    /clients/{id}/transfer-info      → transfer stats
//! - GET    /clients/{id}/torrent-files/{hash} → torrent file list
//! - POST   /clients/{id}/file-priority      → set file priority
//!
//! Static:
//! - GET    /assets/{*path}                  → embedded UI assets

use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};
use tokimo_bus_protocol::{BusListener, DataPlaneSocket};
use tracing::{error, info};

use crate::assets;
use tokimo_app_pt_subscription::{handlers, AppState, subscriptions};

pub fn spawn(service: &str, ctx: Arc<AppState>) -> anyhow::Result<DataPlaneSocket> {
    let (listener, socket) = BusListener::bind_for_app(service)?;
    info!(?socket, "pt-subscription: app server listening");

    let router = build_router(ctx);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            error!(error = %e, "pt-subscription: app server stopped");
        }
    });

    Ok(socket)
}

fn build_router(ctx: Arc<AppState>) -> Router {
    subscriptions::build_subscriptions_app_routes()
        // Client CRUD
        .route("/clients", get(handlers::list_clients).post(handlers::create_client))
        .route(
            "/clients/{id}",
            get(handlers::get_client)
                .put(handlers::update_client)
                .delete(handlers::delete_client),
        )
        .route("/clients/{id}/set-default", post(handlers::set_default))
        .route("/clients/reorder", post(handlers::reorder))
        .route("/clients/{id}/test-connection", get(handlers::test_connection))
        .route("/clients/all-status", get(handlers::get_all_status))
        // Torrent operations
        .route(
            "/clients/{id}/torrents",
            get(handlers::get_torrents).post(handlers::add_torrent),
        )
        .route("/clients/{id}/torrents/pause", post(handlers::pause_torrents))
        .route("/clients/{id}/torrents/resume", post(handlers::resume_torrents))
        .route(
            "/clients/{id}/torrents",
            delete(handlers::delete_torrents),
        )
        .route("/clients/{id}/transfer-info", get(handlers::get_transfer_info))
        .route(
            "/clients/{id}/torrent-files/{hash}",
            get(handlers::get_torrent_files),
        )
        .route("/clients/{id}/file-priority", post(handlers::set_file_priority))
        // Torrent preview and filtered download
        .route("/torrent/preview", post(handlers::preview_torrent_files))
        .route("/torrent/resolve-path", post(handlers::resolve_save_path))
        .route("/torrent/download-filtered", post(handlers::download_with_filter))
        // Static assets
        .route("/assets/{*path}", get(assets::serve))
        .with_state(ctx)
}
