use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterOptions {
    pub sources: Vec<&'static str>,
    pub resolutions: Vec<&'static str>,
    pub codecs: Vec<&'static str>,
}

pub const SOURCES: &[&str] = &["BluRay", "WEB-DL", "WEBRip", "HDTV", "DVD", "Remux", "HDRip"];
pub const RESOLUTIONS: &[&str] = &["480p", "720p", "1080i", "1080p", "2160p", "4320p"];
pub const CODECS: &[&str] = &["H.264", "H.265", "HEVC", "AV1", "VC-1", "MPEG-2"];

pub fn get_filter_options() -> FilterOptions {
    FilterOptions {
        sources: SOURCES.to_vec(),
        resolutions: RESOLUTIONS.to_vec(),
        codecs: CODECS.to_vec(),
    }
}

/// Resolve download path for a given category from a download client's configured paths.
///
/// 1. Find a path where `type == category` and path is non-empty
/// 2. Fallback: find `type == "global"` path and append category as subdirectory
/// 3. Returns None if no suitable path found
pub fn resolve_download_path(
    download_paths: &[(String, String, String)], // (type, path, description)
    category: &str,
) -> Option<String> {
    // 1. Exact category match
    if let Some((_, path, _)) = download_paths.iter().find(|(t, p, _)| t == category && !p.is_empty()) {
        return Some(path.clone());
    }
    // 2. Fallback to global + category
    if let Some((_, global_path, _)) = download_paths.iter().find(|(t, _, _)| t == "global")
        && !global_path.is_empty()
    {
        return Some(format!("{}/{}", global_path.trim_end_matches('/'), category));
    }
    None
}
