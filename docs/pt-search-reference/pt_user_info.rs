use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, warn};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Minimal site credentials needed for user info fetching.
/// Rust-server converts `PtSiteDto` into this before calling.
#[derive(Debug, Clone)]
pub struct PtSiteInput {
    pub site_id: String,
    pub domain: String,
    pub auth_type: String,
    pub cookies: Option<String>,
    pub api_key: Option<String>,
}

/// User info returned from a PT site.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtUserInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploaded: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seeding: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leeching: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vip_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bonus: Option<String>,
}

impl PtUserInfo {
    pub fn empty() -> Self {
        Self {
            uid: None,
            username: None,
            uploaded: None,
            downloaded: None,
            share_ratio: None,
            seeding: None,
            leeching: None,
            vip_group: None,
            bonus: None,
        }
    }
}

/// Fetch user info from a PT site based on its type.
pub async fn fetch_user_info(site: &PtSiteInput) -> Result<PtUserInfo, String> {
    match site.site_id.as_str() {
        "m-team" => fetch_mteam_user_info(site).await,
        _ => Ok(PtUserInfo::empty()),
    }
}

// ── M-Team (API-based) ───────────────────────────────────────────────────────

async fn fetch_mteam_user_info(site: &PtSiteInput) -> Result<PtUserInfo, String> {
    let api_key = site
        .api_key
        .as_deref()
        .ok_or_else(|| "M-Team 需要 API Key".to_string())?;

    let client = build_client()?;
    let base_url = site.domain.trim_end_matches('/');

    let profile_url = format!("{base_url}/api/member/profile");
    debug!(site_id = "m-team", url = %profile_url, "fetching user profile");

    let resp = client
        .post(&profile_url)
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("User-Agent", USER_AGENT)
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("请求用户信息失败: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("获取用户信息失败: HTTP {}", resp.status()));
    }

    let json: Value = resp.json().await.map_err(|e| format!("解析响应失败: {e}"))?;

    let code = json.get("code").and_then(|v| v.as_str()).unwrap_or("-1");
    if code != "0" && code != "SUCCESS" {
        let msg = json.get("message").and_then(|v| v.as_str()).unwrap_or("未知错误");
        return Err(format!("API 错误: {msg}"));
    }

    let data = json.get("data").ok_or("响应缺少 data 字段")?;
    let uid = get_str(data, "id");
    let username = get_str(data, "username");
    let member_count = data.get("memberCount").unwrap_or(&Value::Null);

    let uploaded_bytes = get_str(member_count, "uploaded").parse::<u64>().unwrap_or(0);
    let downloaded_bytes = get_str(member_count, "downloaded").parse::<u64>().unwrap_or(0);
    let share_rate = get_number(member_count, "shareRate");
    let bonus = get_number(member_count, "bonus");

    let (seeding, leeching) = if uid.is_empty() {
        (None, None)
    } else {
        fetch_mteam_peer_status(&client, base_url, api_key, &uid).await
    };

    Ok(PtUserInfo {
        uid: non_empty(uid),
        username: non_empty(username),
        uploaded: Some(format_bytes(uploaded_bytes)),
        downloaded: Some(format_bytes(downloaded_bytes)),
        share_ratio: share_rate.map(|r| format!("{r:.3}")),
        seeding,
        leeching,
        vip_group: None,
        bonus: bonus.map(|b| format!("{b:.1}")),
    })
}

async fn fetch_mteam_peer_status(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    uid: &str,
) -> (Option<i64>, Option<i64>) {
    let url = format!("{base_url}/api/tracker/myPeerStatus");
    let body = serde_json::json!({ "uid": uid });

    let resp = match client
        .post(&url)
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("User-Agent", USER_AGENT)
        .json(&body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!("获取做种状态失败: {e}");
            return (None, None);
        }
    };

    let json: Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => {
            warn!("解析做种状态失败: {e}");
            return (None, None);
        }
    };

    let data = json.get("data").unwrap_or(&Value::Null);
    let seeder = get_str(data, "seeder").parse::<i64>().ok();
    let leecher = get_str(data, "leecher").parse::<i64>().ok();
    (seeder, leecher)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())
}

fn get_str(value: &Value, key: &str) -> String {
    match value.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

fn get_number(value: &Value, key: &str) -> Option<f64> {
    match value.get(key) {
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::String(s)) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn non_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

fn format_bytes(bytes: u64) -> String {
    const TB: f64 = 1_099_511_627_776.0;
    const GB: f64 = 1_073_741_824.0;
    const MB: f64 = 1_048_576.0;

    let b = bytes as f64;
    if b >= TB {
        format!("{:.2} TB", b / TB)
    } else if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else {
        format!("{bytes} B")
    }
}
