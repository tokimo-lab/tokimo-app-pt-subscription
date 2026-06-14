use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::sync::Arc;

use crate::AppState;
use crate::handlers::ok;
use crate::shared::categories::all_categories;
use crate::shared::filter_options::get_filter_options;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryDto {
    pub id: i32,
    pub name: &'static str,
    pub en_name: &'static str,
}

// GET /subscriptions/categories
pub async fn list_categories(_state: State<Arc<AppState>>) -> Response {
    let categories: Vec<CategoryDto> = all_categories()
        .into_iter()
        .map(|(id, en_name, name)| CategoryDto { id, name, en_name })
        .collect();

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Resp {
        categories: Vec<CategoryDto>,
    }

    ok(Resp { categories }).into_response()
}

// GET /subscriptions/filter-options
pub async fn list_filter_options(_state: State<Arc<AppState>>) -> Response {
    ok(get_filter_options()).into_response()
}
