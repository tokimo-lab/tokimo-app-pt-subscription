use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

use super::handlers;
use crate::AppState;

pub fn build_subscriptions_app_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Categories
        .route("/subscriptions/categories", get(handlers::categories::list_categories))
        .route("/subscriptions/filter-options", get(handlers::categories::list_filter_options))
        // PT Search
        .route("/subscriptions/search/pt", post(handlers::search::search_pt))
        // PT Sites
        .route("/subscriptions/pt-sites", get(handlers::pt_sites::list_sites))
        .route("/subscriptions/pt-sites/available", get(handlers::pt_sites::get_available_sites_handler))
        .route("/subscriptions/pt-sites/with-status", get(handlers::pt_sites::list_with_status))
        .route("/subscriptions/pt-sites/all-status", get(handlers::pt_sites::all_status))
        .route("/subscriptions/pt-sites/reorder", post(handlers::pt_sites::reorder_sites))
        .route("/subscriptions/pt-sites/{id}", get(handlers::pt_sites::get_site))
        .route("/subscriptions/pt-sites/{id}", put(handlers::pt_sites::update_site))
        .route("/subscriptions/pt-sites/{id}", delete(handlers::pt_sites::delete_site))
        .route("/subscriptions/pt-sites/{id}/status", get(handlers::pt_sites::get_site_status))
        .route("/subscriptions/pt-sites", post(handlers::pt_sites::create_site))
        // Subscriptions
        .route("/subscriptions", get(handlers::subscription::list))
        .route("/subscriptions", post(handlers::subscription::create))
        .route("/subscriptions/{id}", get(handlers::subscription::get_by_id))
        .route("/subscriptions/{id}", put(handlers::subscription::update))
        .route("/subscriptions/{id}", delete(handlers::subscription::delete))
        .route("/subscriptions/{id}/execute", post(handlers::subscription::execute))
        .route("/subscriptions/{id}/active-run", get(handlers::subscription::get_active_run_id))
        .route("/subscriptions/{id}/debug", get(handlers::subscription::get_debug_info))
        .route("/subscriptions/{id}/logs", get(handlers::subscription::get_recent_logs))
        .route("/subscriptions/{id}/logs/raw", get(handlers::subscription::get_raw_logs))
        .route("/subscriptions/{id}/runs/{run_id}/logs", get(handlers::subscription::get_run_logs))
        .route("/subscriptions/{id}/episode-progress", get(handlers::subscription::get_episode_progress))
        // Traffic management
        .route("/subscriptions/traffic/settings", get(handlers::traffic_manage::get_settings))
        .route("/subscriptions/traffic/settings", put(handlers::traffic_manage::update_settings))
        .route("/subscriptions/traffic/logs", get(handlers::traffic_manage::get_logs))
        .route("/subscriptions/traffic/stats", get(handlers::traffic_manage::get_stats))
        .route("/subscriptions/traffic/trigger-scan", post(handlers::traffic_manage::trigger_scan))
        .route("/subscriptions/traffic/trigger-cleanup", post(handlers::traffic_manage::trigger_cleanup))
}
