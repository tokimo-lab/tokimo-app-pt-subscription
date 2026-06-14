use regex::Regex;
use std::sync::LazyLock;

static SE_EP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)S(\d{1,4})(?:\s*E(\d{1,4})(?:\s*[-~]\s*E?(\d{1,4}))?)?").expect("invalid SE_EP_RE")
});

#[derive(Debug, Clone, PartialEq)]
pub struct EpisodeInfo {
    pub season: Option<i32>,
    pub episodes: Vec<i32>,
    pub is_complete: bool,
}

/// Parse episode info from a file name like "S01E01.mkv" or "Season 1 Complete"
pub fn parse_episodes_from_filename(filename: &str) -> EpisodeInfo {
    let lower = filename.to_lowercase();
    let is_complete = lower.contains("complete") || lower.contains("合集");

    let mut season: Option<i32> = None;
    let mut episodes: Vec<i32> = Vec::new();

    for cap in SE_EP_RE.captures_iter(filename) {
        if let Some(s) = cap.get(1) {
            if let Ok(s_num) = s.as_str().parse::<i32>() {
                season = Some(s_num);
            }
        }
        if let Some(e) = cap.get(2) {
            if let Ok(e_start) = e.as_str().parse::<i32>() {
                if let Some(e_end) = cap.get(3) {
                    if let Ok(e_end_num) = e_end.as_str().parse::<i32>() {
                        for ep in e_start..=e_end_num {
                            if !episodes.contains(&ep) {
                                episodes.push(ep);
                            }
                        }
                    }
                }
                if !episodes.contains(&e_start) {
                    episodes.push(e_start);
                }
            }
        }
    }

    episodes.sort();
    EpisodeInfo {
        season,
        episodes,
        is_complete,
    }
}

/// Check if a file should be included based on subscription's season/episode filter.
/// Returns true if the file matches the filter (should be downloaded).
pub fn should_include_file(
    filename: &str,
    filter_season: Option<i32>,
    filter_episodes: &[i32],
) -> bool {
    let info = parse_episodes_from_filename(filename);

    // If no filter specified, include everything
    if filter_season.is_none() && filter_episodes.is_empty() {
        return true;
    }

    // "Complete" or "合集" files always match the season
    if info.is_complete {
        if let Some(fs) = filter_season {
            return info.season.map_or(true, |s| s == fs);
        }
        return true;
    }

    // Check season match
    if let Some(fs) = filter_season {
        if let Some(file_season) = info.season {
            if file_season != fs {
                return false;
            }
        }
        // If file doesn't have season info, assume it matches
    }

    // Check episode match
    if !filter_episodes.is_empty() {
        if info.episodes.is_empty() {
            // File has no episode info - include it (could be subtitle, extra, etc.)
            return true;
        }
        // Include if ANY episode in the file matches the filter
        return info.episodes.iter().any(|ep| filter_episodes.contains(ep));
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_season_episode() {
        let info = parse_episodes_from_filename("The.Long.Season.S01E01.2160p.WEB-DL.mkv");
        assert_eq!(info.season, Some(1));
        assert_eq!(info.episodes, vec![1]);
        assert!(!info.is_complete);
    }

    #[test]
    fn test_parse_episode_range() {
        let info = parse_episodes_from_filename("Show.S01E01-E10.1080p.mkv");
        assert_eq!(info.season, Some(1));
        assert_eq!(info.episodes, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_parse_complete() {
        let info = parse_episodes_from_filename("Show.S01.Complete.2160p.mkv");
        assert_eq!(info.season, Some(1));
        assert!(info.is_complete);
    }

    #[test]
    fn test_should_include() {
        assert!(should_include_file("S01E01.mkv", Some(1), &[1, 2, 3]));
        assert!(!should_include_file("S01E05.mkv", Some(1), &[1, 2, 3]));
        assert!(!should_include_file("S02E01.mkv", Some(1), &[1, 2, 3]));
        assert!(should_include_file("S01.Complete.mkv", Some(1), &[]));
    }
}
