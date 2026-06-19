//! Canonical media-category taxonomy — the single source of truth.
//!
//! Every PT site maps its raw category codes onto this taxonomy. A category is
//! identified by a stable string `slug` (e.g. "movie", "tv"). The backend only
//! ever deals in slugs; all human-readable / localized labels are the
//! frontend's responsibility (i18n).

/// Canonical media category. Identified by a stable string slug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Movie,
    Tv,
    Anime,
    Documentary,
    Variety,
    Sports,
    Music,
    Ebook,
    Audiobook,
    Software,
    Game,
    Course,
    Other,
}

impl Category {
    /// Every canonical category, in display order.
    pub const ALL: [Category; 13] = [
        Category::Movie,
        Category::Tv,
        Category::Anime,
        Category::Documentary,
        Category::Variety,
        Category::Sports,
        Category::Music,
        Category::Ebook,
        Category::Audiobook,
        Category::Software,
        Category::Game,
        Category::Course,
        Category::Other,
    ];

    /// Stable slug — the canonical identity used for storage and filtering.
    pub fn slug(self) -> &'static str {
        match self {
            Category::Movie => "movie",
            Category::Tv => "tv",
            Category::Anime => "anime",
            Category::Documentary => "documentary",
            Category::Variety => "variety",
            Category::Sports => "sports",
            Category::Music => "music",
            Category::Ebook => "ebook",
            Category::Audiobook => "audiobook",
            Category::Software => "software",
            Category::Game => "game",
            Category::Course => "course",
            Category::Other => "other",
        }
    }

    /// Resolve a category from its slug.
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.slug() == slug)
    }
}

/// Map a site's raw category code onto the canonical taxonomy.
///
/// Returns `None` when the site/code pair is unknown; callers typically fall
/// back to [`Category::Other`].
pub fn map_site_category(site_id: &str, raw_category: &str) -> Option<Category> {
    match site_id {
        "m-team" => mapping::mteam(raw_category),
        "acgrip" | "mikanani" => Some(Category::Anime),
        "sukebei" => mapping::sukebei(raw_category),
        _ if mapping::is_nexus_site(site_id) => mapping::nexus(raw_category),
        _ => None,
    }
}

/// Resolve a raw site category to a canonical slug, defaulting to `"other"`.
pub fn resolve_category(site_id: &str, raw_category: &str) -> String {
    map_site_category(site_id, raw_category)
        .unwrap_or(Category::Other)
        .slug()
        .to_string()
}

/// All canonical category slugs, in display order.
pub fn all_categories() -> Vec<&'static str> {
    Category::ALL.iter().map(|c| c.slug()).collect()
}

/// Per-site → canonical category mapping tables.
///
/// Each site family has its own clearly separated table. To support a new site,
/// add it to [`is_nexus_site`] (if it follows NexusPHP numbering) or add a new
/// dedicated mapping function and wire it up in [`map_site_category`].
mod mapping {
    use super::Category;

    /// NexusPHP-family sites that share the standard NexusPHP category numbering.
    pub fn is_nexus_site(site_id: &str) -> bool {
        matches!(
            site_id,
            "hdfans"
                | "hdsky"
                | "audiences"
                | "azusa"
                | "btschool"
                | "chdbits"
                | "hddolby"
                | "HDHome"
                | "hhan"
                | "keepfrds"
                | "ourbits"
                | "pterclub"
                | "ptsbao"
                | "putao"
                | "ssd"
                | "ultrahd"
                | "tjupt"
                | "ttg"
                | "hares"
                | "hdatmos"
                | "agsv"
                | "filelist"
                | "iptorrents"
                | "exoticaz"
        )
    }

    /// M-Team category mapping (API returns numeric IDs like "400", "401").
    pub fn mteam(raw: &str) -> Option<Category> {
        match raw {
            "400" | "401" | "419" => Some(Category::Movie),
            "402" => Some(Category::Tv),
            "403" | "405" | "433" | "440" => Some(Category::Anime),
            "404" | "441" => Some(Category::Documentary),
            "406" | "429" => Some(Category::Variety),
            "407" | "425" | "439" => Some(Category::Sports),
            "408" | "430" | "434" | "435" | "443" => Some(Category::Music),
            "426" | "427" => Some(Category::Ebook),
            "428" | "437" | "442" => Some(Category::Audiobook),
            "423" | "431" => Some(Category::Software),
            "432" => Some(Category::Game),
            "420" | "438" => Some(Category::Course),
            "421" | "409" | "436" => Some(Category::Other),
            _ => None,
        }
    }

    /// Standard NexusPHP category mapping (numeric category IDs).
    pub fn nexus(raw: &str) -> Option<Category> {
        match raw {
            "401" | "419" | "420" | "421" | "422" | "423" | "424" | "425" => Some(Category::Movie),
            "402" | "426" | "427" | "428" | "429" => Some(Category::Tv),
            "403" | "430" | "431" | "432" | "433" => Some(Category::Anime),
            "404" | "434" | "435" => Some(Category::Documentary),
            "405" | "436" | "437" => Some(Category::Variety),
            "406" | "438" | "439" | "440" => Some(Category::Sports),
            "407" | "408" | "441" | "442" | "443" => Some(Category::Music),
            "410" | "444" | "445" => Some(Category::Software),
            "411" | "446" | "447" => Some(Category::Game),
            "412" | "448" | "449" => Some(Category::Course),
            "413" | "450" | "451" => Some(Category::Ebook),
            "414" | "452" | "453" => Some(Category::Audiobook),
            "409" | "415" | "416" | "417" | "418" => Some(Category::Other),
            _ => None,
        }
    }

    /// Sukebei (adult) category mapping.
    #[allow(clippy::unnecessary_wraps)]
    pub fn sukebei(raw: &str) -> Option<Category> {
        match raw {
            "4" | "15" | "16" | "17" => Some(Category::Movie),
            "6" => Some(Category::Anime),
            "7" => Some(Category::Game),
            _ => Some(Category::Other),
        }
    }
}
