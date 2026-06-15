use tokimo_pt_search::pt_search::PtSearchResult;

use crate::db::entities::subscriptions;

pub struct FilterPrefs {
    pub sources: Option<Vec<String>>,
    pub resolutions: Option<Vec<String>>,
    pub codecs: Option<Vec<String>>,
    pub free_only: bool,
    pub exclude_hr: bool,
    pub include_keywords: Option<String>,
    pub exclude_keywords: Option<String>,
    pub min_size: f64,
    pub max_size: f64,
    pub min_seeders: f64,
    pub max_seeders: f64,
}

impl FilterPrefs {
    pub fn from_model(sub: &subscriptions::Model) -> Self {
        let json_to_vec = |v: &Option<serde_json::Value>| -> Option<Vec<String>> {
            v.as_ref().and_then(|j| serde_json::from_value(j.clone()).ok())
        };
        Self {
            sources: json_to_vec(&sub.sources),
            resolutions: json_to_vec(&sub.resolutions),
            codecs: json_to_vec(&sub.codecs),
            free_only: sub.free_only,
            exclude_hr: sub.exclude_hr,
            include_keywords: sub.include_keywords.clone(),
            exclude_keywords: sub.exclude_keywords.clone(),
            min_size: sub.min_size.parse().unwrap_or(0.0),
            max_size: sub.max_size.parse().unwrap_or(0.0),
            min_seeders: sub.min_seeders.parse().unwrap_or(0.0),
            max_seeders: sub.max_seeders.parse().unwrap_or(0.0),
        }
    }
}

pub fn should_exclude(torrent: &PtSearchResult, prefs: &FilterPrefs) -> bool {
    // free_only
    if prefs.free_only && torrent.discount.as_deref() != Some("FREE") {
        return true;
    }

    // exclude_hr
    if prefs.exclude_hr {
        let lower = torrent.title.to_lowercase();
        if lower.contains(" hr ") || lower.starts_with("hr ") || lower.ends_with(" hr") {
            return true;
        }
    }

    // exclude_keywords
    if let Some(ref kw) = prefs.exclude_keywords {
        let lower_title = torrent.title.to_lowercase();
        for word in kw.split(',').map(|s| s.trim().to_lowercase()) {
            if !word.is_empty() && lower_title.contains(&word) {
                return true;
            }
        }
    }

    // include_keywords (must match at least one)
    if let Some(ref kw) = prefs.include_keywords {
        let lower_title = torrent.title.to_lowercase();
        let matched = kw
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .any(|w| !w.is_empty() && lower_title.contains(&w));
        if !matched {
            return true;
        }
    }

    // sources filter
    if let Some(ref allowed) = prefs.sources
        && let Some(ref src) = torrent.source
    {
        let lower = src.to_lowercase();
        if !allowed.iter().any(|a| lower.contains(&a.to_lowercase())) {
            return true;
        }
    }

    // resolutions filter
    if let Some(ref allowed) = prefs.resolutions
        && let Some(ref res) = torrent.resolution
    {
        let norm = normalize_resolution(res);
        if !allowed.iter().any(|a| normalize_resolution(a) == norm) {
            return true;
        }
    }

    // codecs filter
    if let Some(ref allowed) = prefs.codecs
        && let Some(ref codec) = torrent.video_codec
    {
        let lower = codec.to_lowercase();
        if !allowed.iter().any(|a| a.to_lowercase() == lower) {
            return true;
        }
    }

    // size range
    let size_gb = torrent.size_bytes.unwrap_or(0) as f64 / 1_073_741_824.0;
    if prefs.min_size > 0.0 && size_gb < prefs.min_size {
        return true;
    }
    if prefs.max_size > 0.0 && size_gb > prefs.max_size {
        return true;
    }

    // seeders range
    let seeders = f64::from(torrent.seeders);
    if prefs.min_seeders > 0.0 && seeders < prefs.min_seeders {
        return true;
    }
    if prefs.max_seeders > 0.0 && seeders > prefs.max_seeders {
        return true;
    }

    // exclude obvious fakes
    let lower = torrent.title.to_lowercase();
    if lower.contains("cam") || lower.contains("hdcam") || lower.contains(" ts ") {
        return true;
    }

    false
}

pub fn score_torrent(torrent: &PtSearchResult, prefs: &FilterPrefs) -> f64 {
    if should_exclude(torrent, prefs) {
        return -1.0;
    }

    let mut score = 1.0;

    // ── Resolution ──
    score += match torrent.resolution.as_deref() {
        Some("2160p") => 30.0,
        Some("1080p") => 25.0,
        Some("720p") => 15.0,
        Some("480p") => 5.0,
        _ => 10.0,
    };
    if let Some(ref allowed) = prefs.resolutions
        && let Some(ref res) = torrent.resolution
    {
        let norm = normalize_resolution(res);
        if allowed.iter().any(|a| normalize_resolution(a) == norm) {
            score += 10.0;
        }
    }

    // ── Video codec ──
    score += match torrent.video_codec.as_deref() {
        Some("HEVC" | "H.265") => 25.0,
        Some("H.264" | "AVC") => 18.0,
        Some("AV1") => 20.0,
        _ => 10.0,
    };
    if let Some(ref allowed) = prefs.codecs
        && let Some(ref codec) = torrent.video_codec
    {
        let lower = codec.to_lowercase();
        if allowed.iter().any(|a| a.to_lowercase() == lower) {
            score += 10.0;
        }
    }

    // ── Source ──
    score += match torrent.source.as_deref() {
        Some("BluRay" | "REMUX") => 15.0,
        Some("WEB-DL" | "WEBDL") => 12.0,
        Some("WEBRip") => 10.0,
        Some("HDTV") => 5.0,
        _ => 8.0,
    };
    if let Some(ref allowed) = prefs.sources
        && let Some(ref src) = torrent.source
    {
        let lower = src.to_lowercase();
        if allowed.iter().any(|a| lower.contains(&a.to_lowercase())) {
            score += 5.0;
        }
    }

    // ── Seeders (log scale) ──
    let seeders = f64::from(torrent.seeders).max(1.0);
    score += (seeders.log10() * 5.0).min(15.0);

    // ── Free bonus ──
    if torrent.discount.as_deref() == Some("FREE") {
        score += 20.0;
    }

    // ── Size reasonableness ──
    let size_gb = torrent.size_bytes.unwrap_or(0) as f64 / 1_073_741_824.0;
    if size_gb > 0.5 && size_gb < 50.0 {
        score += 10.0;
    }
    if size_gb > 80.0 {
        score -= 5.0;
    }

    score
}

pub fn rank_torrents(torrents: &[PtSearchResult], prefs: &FilterPrefs, max_count: usize) -> Vec<(usize, f64)> {
    let mut scored: Vec<(usize, f64)> = torrents
        .iter()
        .enumerate()
        .map(|(i, t)| (i, score_torrent(t, prefs)))
        .filter(|(_, s)| *s >= 0.0)
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(max_count);
    scored
}

fn normalize_resolution(res: &str) -> String {
    match res.to_lowercase().as_str() {
        "4k" | "uhd" | "2160p" => "2160p".into(),
        "1080p" | "fhd" => "1080p".into(),
        "720p" | "hd" => "720p".into(),
        "480p" | "sd" => "480p".into(),
        "8k" | "4320p" => "4320p".into(),
        other => other.into(),
    }
}
