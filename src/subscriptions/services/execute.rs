use std::sync::Arc;

use sea_orm::*;
use tracing::error;

use crate::AppState;
use crate::db::entities::subscriptions;
use crate::db::repos::download_client_repo::DownloadClientRepo;
use crate::services::DownloadClientService;
use crate::shared::categories::category_to_en_name;
use crate::shared::episode_parser::should_include_file;
use crate::shared::filter_options::resolve_download_path;
use crate::shared::torrent_parser::parse_torrent;
use crate::subscriptions::handlers::subscription as sub_handler;
use crate::subscriptions::models::pt_site::PtSiteDto;
use crate::subscriptions::repos::pt_site_repo::PtSiteRepo;
use crate::subscriptions::repos::subscription_repo::SubscriptionRepo;
use crate::subscriptions::services::pt_search::search_all_sites;
use crate::subscriptions::services::scorer::{FilterPrefs, rank_torrents};
use tokimo_package_client_api::downloaders::traits::AddTorrentOptions;

fn build_search_keywords(sub: &subscriptions::Model) -> Vec<String> {
    let title = &sub.title;
    if sub.media_type == "movie" || sub.season.is_none() {
        let mut kw = title.clone();
        if let Some(ref year) = sub.year {
            kw.push(' ');
            kw.push_str(year);
        }
        return vec![kw];
    }
    let season_num: i32 = sub.season.as_ref().and_then(|s| s.parse().ok()).unwrap_or(1);
    let season_str = format!("S{:02}", season_num);
    vec![format!("{title} {season_str}")]
}

pub async fn execute_subscription(state: &Arc<AppState>, sub_id: &str) -> bool {
    let storage = Arc::clone(state.storage());

    let sub = match SubscriptionRepo::get_raw(&state.db, sub_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return false,
        Err(e) => {
            error!("execute: get subscription failed: {e}");
            return false;
        }
    };

    let run_id = uuid::Uuid::new_v4().to_string();
    sub_handler::append_log(
        &storage,
        sub_id,
        &run_id,
        "start",
        &format!("开始执行: {}", sub.title),
        None,
    )
    .await;

    // Update last_checked_at at execution start, regardless of outcome
    update_timestamps(state, sub_id, &sub).await;

    let keywords = build_search_keywords(&sub);
    sub_handler::append_log(
        &storage,
        sub_id,
        &run_id,
        "searching",
        &format!("关键词: {}", keywords.join(", ")),
        None,
    )
    .await;

    let sites = match get_search_sites(&state.db, &sub).await {
        Ok(s) => s,
        Err(e) => {
            sub_handler::append_log(&storage, sub_id, &run_id, "error", &format!("获取站点失败: {e}"), None).await;
            return false;
        }
    };

    if sites.is_empty() {
        sub_handler::append_log(&storage, sub_id, &run_id, "completed", "无可用 PT 站点", None).await;
        return false;
    }

    let filter_categories: Vec<String> = sub.category.as_ref().map(|c| vec![c.clone()]).unwrap_or_default();

    let mut all_results: Vec<(tokimo_pt_search::pt_search::PtSearchResult, String)> = Vec::new();
    for keyword in &keywords {
        let resp = search_all_sites(&state.http_client, &sites, keyword, &filter_categories).await;

        // Log per-site results for debugging
        for summary in &resp.site_summaries {
            sub_handler::append_log(
                &storage,
                sub_id,
                &run_id,
                "searching",
                &format!("{}: {} 个结果", summary.site_name, summary.count),
                Some(serde_json::json!({ "siteId": summary.site_db_id, "count": summary.count })),
            )
            .await;
        }

        for r in resp.results {
            if !all_results
                .iter()
                .any(|(x, _)| x.id == r.result.id && x.title == r.result.title)
            {
                all_results.push((r.result, r.site_db_id));
            }
        }
    }

    sub_handler::append_log(
        &storage,
        sub_id,
        &run_id,
        "searching",
        &format!("搜索到 {} 个种子", all_results.len()),
        None,
    )
    .await;

    if all_results.is_empty() {
        sub_handler::append_log(&storage, sub_id, &run_id, "completed", "未找到种子", None).await;
        return false;
    }

    let prefs = FilterPrefs::from_model(&sub);
    let ranked = rank_torrents(
        &all_results.iter().map(|(r, _)| r.clone()).collect::<Vec<_>>(),
        &prefs,
        sub.max_downloads_per_run as usize,
    );

    sub_handler::append_log(
        &storage,
        sub_id,
        &run_id,
        "filtering",
        &format!("过滤后 {} 个候选", ranked.len()),
        None,
    )
    .await;

    if ranked.is_empty() {
        sub_handler::append_log(&storage, sub_id, &run_id, "completed", "过滤后无匹配种子", None).await;
        return false;
    }

    let (best_idx, best_score) = ranked[0];
    let (best, best_site_id) = &all_results[best_idx];

    sub_handler::append_log(
        &storage, sub_id, &run_id, "matching",
        &format!("最佳匹配: {} (分数 {:.1})", best.title, best_score),
        Some(serde_json::json!({ "title": best.title, "score": best_score, "seeders": best.seeders, "detailUrl": best.detail_url })),
    ).await;

    if !best.detail_url.is_empty() {
        sub_handler::append_log(
            &storage,
            sub_id,
            &run_id,
            "matching",
            &format!("链接: {}", best.detail_url),
            None,
        )
        .await;
    }

    // Resolve download path
    let save_path = if let Some(ref client_id) = sub.download_client_id {
        let cid = client_id.to_string();
        match DownloadClientRepo::get_by_id(&state.db, &cid).await {
            Ok(Some(client)) => {
                let paths: Vec<(String, String, String)> = client
                    .download_paths
                    .iter()
                    .map(|p| (p.r#type.clone(), p.path.clone(), p.description.clone()))
                    .collect();
                let cat_raw = sub.category.as_deref().unwrap_or("global");
                let cat = category_to_en_name(cat_raw);
                resolve_download_path(&paths, &cat)
            }
            _ => None,
        }
    } else {
        None
    };

    // Find download client
    let client_id = if let Some(ref cid) = sub.download_client_id {
        let cid_str = cid.to_string();
        // Verify the configured client exists
        match DownloadClientRepo::get_by_id(&state.db, &cid_str).await {
            Ok(Some(client)) => {
                sub_handler::append_log(
                    &storage, sub_id, &run_id, "downloading",
                    &format!("使用配置的下载器: {}", client.name),
                    Some(serde_json::json!({ "clientId": client.id, "clientName": client.name, "clientType": client.r#type })),
                ).await;
                cid.to_string()
            }
            Ok(None) => {
                sub_handler::append_log(
                    &storage,
                    sub_id,
                    &run_id,
                    "error",
                    &format!("配置的下载器不存在: {cid}，尝试使用默认下载器"),
                    Some(serde_json::json!({ "configuredClientId": cid })),
                )
                .await;
                // Fall through to find default
                match DownloadClientRepo::list(&state.db).await {
                    Ok(clients) => {
                        if let Some(default) = clients.iter().find(|c| c.is_default) {
                            sub_handler::append_log(
                                &storage,
                                sub_id,
                                &run_id,
                                "downloading",
                                &format!("使用默认下载器: {}", default.name),
                                Some(serde_json::json!({ "clientId": default.id, "clientName": default.name })),
                            )
                            .await;
                            default.id.clone()
                        } else if let Some(first) = clients.first() {
                            sub_handler::append_log(
                                &storage,
                                sub_id,
                                &run_id,
                                "downloading",
                                &format!("使用第一个可用下载器: {}", first.name),
                                Some(serde_json::json!({ "clientId": first.id, "clientName": first.name })),
                            )
                            .await;
                            first.id.clone()
                        } else {
                            sub_handler::append_log(&storage, sub_id, &run_id, "error", "无可用下载器", None).await;
                            return false;
                        }
                    }
                    Err(e) => {
                        sub_handler::append_log(
                            &storage,
                            sub_id,
                            &run_id,
                            "error",
                            &format!("获取下载器失败: {e}"),
                            None,
                        )
                        .await;
                        return false;
                    }
                }
            }
            Err(e) => {
                sub_handler::append_log(
                    &storage,
                    sub_id,
                    &run_id,
                    "error",
                    &format!("查询下载器失败: {e}"),
                    None,
                )
                .await;
                return false;
            }
        }
    } else {
        match DownloadClientRepo::list(&state.db).await {
            Ok(clients) => {
                if let Some(default) = clients.iter().find(|c| c.is_default) {
                    sub_handler::append_log(
                        &storage,
                        sub_id,
                        &run_id,
                        "downloading",
                        &format!("使用默认下载器: {}", default.name),
                        Some(serde_json::json!({ "clientId": default.id, "clientName": default.name })),
                    )
                    .await;
                    default.id.clone()
                } else if let Some(first) = clients.first() {
                    sub_handler::append_log(
                        &storage,
                        sub_id,
                        &run_id,
                        "downloading",
                        &format!("使用第一个可用下载器: {}", first.name),
                        Some(serde_json::json!({ "clientId": first.id, "clientName": first.name })),
                    )
                    .await;
                    first.id.clone()
                } else {
                    sub_handler::append_log(&storage, sub_id, &run_id, "error", "无可用下载器", None).await;
                    return false;
                }
            }
            Err(e) => {
                sub_handler::append_log(
                    &storage,
                    sub_id,
                    &run_id,
                    "error",
                    &format!("获取下载器失败: {e}"),
                    None,
                )
                .await;
                return false;
            }
        }
    };

    let download_url = best.download_url.clone();
    if download_url.is_empty() {
        sub_handler::append_log(&storage, sub_id, &run_id, "error", "种子无下载链接", None).await;
        return false;
    }

    sub_handler::append_log(
        &storage,
        sub_id,
        &run_id,
        "downloading",
        &format!("准备推送到下载器: {}", best.title),
        Some(serde_json::json!({
            "downloadUrl": download_url,
            "savePath": save_path,
            "category": sub.category,
            "clientId": client_id
        })),
    )
    .await;

    // For API sites (like M-Team), download torrent file first
    let use_torrent_file = download_url.contains("/api/torrent/genDlToken") || download_url.contains("/api/");

    let options = if use_torrent_file {
        // Download torrent file using API key
        let site = sites.iter().find(|s| s.id == *best_site_id);
        let api_key = site.and_then(|s| s.api_key.clone()).unwrap_or_default();

        sub_handler::append_log(
            &storage,
            sub_id,
            &run_id,
            "downloading",
            "API 站点，先下载种子文件",
            Some(serde_json::json!({ "siteId": best_site_id })),
        )
        .await;

        // Call M-Team API to get download token (requires form-urlencoded, no redirect follow)
        let gen_token_url = format!(
            "{}/api/torrent/genDlToken",
            sites
                .iter()
                .find(|s| s.id == *best_site_id)
                .map(|s| s.domain.trim_end_matches('/'))
                .unwrap_or("https://api.m-team.cc")
        );

        let form_data = format!("id={}", best.id);
        // Build a client that does NOT follow redirects (M-Team returns 302 with JSON body)
        let no_redirect_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_else(|_| state.http_client.clone());

        let req = no_redirect_client
            .post(&gen_token_url)
            .header("x-api-key", &api_key)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_data);

        match req.send().await {
            Ok(resp) => {
                let status = resp.status();
                // M-Team may return 200 or 302 with JSON body
                if status.is_success() || status.is_redirection() {
                    match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            // Get the download URL from response
                            let token_url = json.get("data").and_then(|d| d.as_str()).unwrap_or("");

                            if !token_url.is_empty() {
                                // Download the torrent file (needs UA + follow redirects for dlv2)
                                let dl_client = reqwest::Client::builder()
                                    .redirect(reqwest::redirect::Policy::limited(5))
                                    .timeout(std::time::Duration::from_secs(30))
                                    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                                    .build()
                                    .unwrap_or_else(|_| state.http_client.clone());
                                match dl_client.get(token_url).send().await {
                                    Ok(torrent_resp) => {
                                        if torrent_resp.status().is_success() {
                                            match torrent_resp.bytes().await {
                                                Ok(bytes) => {
                                                    use base64::Engine;
                                                    let b64 =
                                                        base64::engine::general_purpose::STANDARD_NO_PAD.encode(&bytes);
                                                    sub_handler::append_log(
                                                        &storage,
                                                        sub_id,
                                                        &run_id,
                                                        "downloading",
                                                        &format!("种子文件下载成功，大小: {} bytes", bytes.len()),
                                                        None,
                                                    )
                                                    .await;

                                                    // Parse torrent to get file list for episode filtering
                                                    let excluded_indices = match parse_torrent(&bytes) {
                                                        Ok(meta) => {
                                                            let filter_season =
                                                                sub.season.as_ref().and_then(|s| s.parse::<i32>().ok());
                                                            let filter_episodes: Vec<i32> = sub
                                                                .episodes
                                                                .as_ref()
                                                                .and_then(|e| {
                                                                    serde_json::from_value::<Vec<i32>>(e.clone()).ok()
                                                                })
                                                                .unwrap_or_default();
                                                            let has_filter =
                                                                filter_season.is_some() || !filter_episodes.is_empty();

                                                            if has_filter {
                                                                let excluded: Vec<u32> = meta
                                                                    .files
                                                                    .iter()
                                                                    .filter(|f| {
                                                                        !should_include_file(
                                                                            &f.path,
                                                                            filter_season,
                                                                            &filter_episodes,
                                                                        )
                                                                    })
                                                                    .map(|f| f.index as u32)
                                                                    .collect();

                                                                if !excluded.is_empty() {
                                                                    sub_handler::append_log(
                                                                        &storage,
                                                                        sub_id,
                                                                        &run_id,
                                                                        "downloading",
                                                                        &format!(
                                                                            "集数过滤: 共 {} 个文件，排除 {} 个",
                                                                            meta.files.len(),
                                                                            excluded.len()
                                                                        ),
                                                                        None,
                                                                    )
                                                                    .await;
                                                                }
                                                                excluded
                                                            } else {
                                                                vec![]
                                                            }
                                                        }
                                                        Err(e) => {
                                                            sub_handler::append_log(
                                                                &storage,
                                                                sub_id,
                                                                &run_id,
                                                                "downloading",
                                                                &format!("解析种子文件失败，跳过集数过滤: {e}"),
                                                                None,
                                                            )
                                                            .await;
                                                            vec![]
                                                        }
                                                    };

                                                    let need_filter = !excluded_indices.is_empty();
                                                    AddTorrentOptions {
                                                        urls: None,
                                                        torrents: Some(vec![b64]),
                                                        save_path: save_path.clone(),
                                                        category: sub.category.clone(),
                                                        tags: Some(vec!["tokimo-subscription".into()]),
                                                        paused: if need_filter { Some(true) } else { Some(false) },
                                                        skip_hash_check: None,
                                                    }
                                                }
                                                Err(e) => {
                                                    sub_handler::append_log(
                                                        &storage,
                                                        sub_id,
                                                        &run_id,
                                                        "error",
                                                        &format!("读取种子文件失败: {e}"),
                                                        None,
                                                    )
                                                    .await;
                                                    return false;
                                                }
                                            }
                                        } else {
                                            sub_handler::append_log(
                                                &storage,
                                                sub_id,
                                                &run_id,
                                                "error",
                                                &format!("下载种子文件失败: HTTP {}", torrent_resp.status()),
                                                None,
                                            )
                                            .await;
                                            return false;
                                        }
                                    }
                                    Err(e) => {
                                        sub_handler::append_log(
                                            &storage,
                                            sub_id,
                                            &run_id,
                                            "error",
                                            &format!("下载种子文件请求失败: {e}"),
                                            None,
                                        )
                                        .await;
                                        return false;
                                    }
                                }
                            } else {
                                sub_handler::append_log(
                                    &storage,
                                    sub_id,
                                    &run_id,
                                    "error",
                                    "API 返回的下载链接为空",
                                    Some(serde_json::json!({ "response": json })),
                                )
                                .await;
                                return false;
                            }
                        }
                        Err(e) => {
                            sub_handler::append_log(
                                &storage,
                                sub_id,
                                &run_id,
                                "error",
                                &format!("解析 API 响应失败: {e}"),
                                None,
                            )
                            .await;
                            return false;
                        }
                    }
                } else {
                    sub_handler::append_log(
                        &storage,
                        sub_id,
                        &run_id,
                        "error",
                        &format!("获取下载令牌失败: HTTP {}", resp.status()),
                        None,
                    )
                    .await;
                    return false;
                }
            }
            Err(e) => {
                sub_handler::append_log(
                    &storage,
                    sub_id,
                    &run_id,
                    "error",
                    &format!("获取下载令牌请求失败: {e}"),
                    None,
                )
                .await;
                return false;
            }
        }
    } else {
        AddTorrentOptions {
            urls: Some(vec![download_url.clone()]),
            torrents: None,
            save_path: save_path.clone(),
            category: sub.category.clone(),
            tags: Some(vec!["tokimo-subscription".into()]),
            paused: Some(false),
            skip_hash_check: None,
        }
    };

    match DownloadClientService::add_torrent(&state.db, &client_id, options, &state.http_client).await {
        Ok(()) => {
            sub_handler::append_log(
                &storage, sub_id, &run_id, "downloading",
                &format!("已推送到下载器: {}", best.title),
                Some(serde_json::json!({ "torrent": best.title, "savePath": save_path, "client": client_id, "downloadUrl": download_url })),
            ).await;

            // Set status to "pushed" - subscription goal achieved
            let uid = uuid::Uuid::parse_str(sub_id).unwrap_or_default();
            if let Ok(Some(model)) = subscriptions::Entity::find_by_id(uid).one(&state.db).await {
                let mut active: subscriptions::ActiveModel = model.into();
                active.status = Set("pushed".to_string());
                // Clear next_check_at - no more polling needed
                active.next_check_at = Set(None);
                let _ = active.update(&state.db).await;
            }

            sub_handler::append_log(&storage, sub_id, &run_id, "completed", "执行完成，已推送下载", None).await;
            // Don't update timestamps - subscription is done
            true
        }
        Err(e) => {
            sub_handler::append_log(&storage, sub_id, &run_id, "error", &format!("推送下载失败: {e}"), None).await;
            false
        }
    }
}

async fn get_search_sites(
    db: &sea_orm::DatabaseConnection,
    sub: &subscriptions::Model,
) -> Result<Vec<PtSiteDto>, crate::AppError> {
    let all_sites = PtSiteRepo::list(db).await?;
    let site_ids: Option<Vec<String>> = sub
        .site_ids
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    if let Some(ref ids) = site_ids {
        if !ids.is_empty() {
            return Ok(all_sites.into_iter().filter(|s| ids.contains(&s.id)).collect());
        }
    }
    Ok(all_sites)
}

async fn update_timestamps(state: &Arc<AppState>, sub_id: &str, sub: &subscriptions::Model) {
    let interval: i32 = sub.interval_minutes.parse().unwrap_or(5);
    if let Err(e) = SubscriptionRepo::update_timestamps(&state.db, sub_id, interval).await {
        error!("update_timestamps failed: {e}");
    }
}
