//! Download-path resolution.
//!
//! Owns all logic for turning a media category + a download client's configured
//! paths into a concrete save path.

use crate::shared::categories::Category;

/// Normalize an incoming category string to a canonical slug for path lookup.
///
/// Accepts a canonical slug (passes through) or any unknown string (returned
/// as-is so callers like `"global"` still work).
pub fn normalize_category_slug(category: &str) -> String {
    Category::from_slug(category).map_or_else(|| category.to_string(), |c| c.slug().to_string())
}

/// Resolve a download path for a given category from a download client's
/// configured paths.
///
/// 1. Find a path where `type == category` and path is non-empty
/// 2. Fallback: find `type == "global"` path and append category as subdirectory
/// 3. Returns `None` if no suitable path found
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
