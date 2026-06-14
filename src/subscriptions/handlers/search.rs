use axum::{Json, extract::State, response::{IntoResponse, Response}};
use serde::Deserialize;
use std::sync::Arc;

use crate::handlers::{ok, user::AuthUser};
use crate::subscriptions::repos::pt_site_repo::PtSiteRepo;
use crate::subscriptions::services::pt_search::search_all_sites;
use crate::{AppError, AppState};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPtInput {
    pub keyword: String,
    #[serde(default)]
    pub site_ids: Vec<String>,
    /// Filter by canonical category IDs (M-Team standard). Empty = all.
    #[serde(default)]
    pub categories: Vec<String>,
}

pub async fn search_pt(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Json(input): Json<SearchPtInput>,
) -> Response {
    let keyword = input.keyword.trim().to_string();
    if keyword.is_empty() {
        return AppError::BadRequest("keyword is empty".into()).into_response();
    }

    let sites = match PtSiteRepo::list(&state.db).await {
        Ok(sites) => sites,
        Err(e) => return e.into_response(),
    };

    if sites.is_empty() {
        return AppError::BadRequest("没有配置 PT 站点".into()).into_response();
    }

    let sites = if input.site_ids.is_empty() {
        sites
    } else {
        sites
            .into_iter()
            .filter(|s| input.site_ids.contains(&s.id))
            .collect()
    };

    let response = search_all_sites(
        &state.http_client,
        &sites,
        &keyword,
        &input.categories,
    )
    .await;

    ok(response).into_response()
}
