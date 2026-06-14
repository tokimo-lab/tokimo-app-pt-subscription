use serde::Serialize;

/// Canonical media category — unified across all PT sites.
/// Integer IDs are stable and used for storage/filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Category {
    Movie = 1,
    Tv = 2,
    Anime = 3,
    Documentary = 4,
    Variety = 5,
    Sports = 6,
    Music = 7,
    Ebook = 8,
    Audiobook = 9,
    Software = 10,
    Game = 11,
    Course = 12,
    Other = 99,
}

impl Category {
    pub fn id(self) -> i32 {
        self as i32
    }

    pub fn name(self) -> &'static str {
        match self {
            Category::Movie => "电影",
            Category::Tv => "剧集",
            Category::Anime => "动漫",
            Category::Documentary => "纪录片",
            Category::Variety => "综艺",
            Category::Sports => "体育",
            Category::Music => "音乐",
            Category::Ebook => "电子书",
            Category::Audiobook => "有声书",
            Category::Software => "软件",
            Category::Game => "游戏",
            Category::Course => "课程",
            Category::Other => "其他",
        }
    }

    pub fn en_name(self) -> &'static str {
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

    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Category::Movie),
            2 => Some(Category::Tv),
            3 => Some(Category::Anime),
            4 => Some(Category::Documentary),
            5 => Some(Category::Variety),
            6 => Some(Category::Sports),
            7 => Some(Category::Music),
            8 => Some(Category::Ebook),
            9 => Some(Category::Audiobook),
            10 => Some(Category::Software),
            11 => Some(Category::Game),
            12 => Some(Category::Course),
            99 => Some(Category::Other),
            _ => None,
        }
    }

    pub fn from_en_name(name: &str) -> Option<Self> {
        match name {
            "movie" => Some(Category::Movie),
            "tv" => Some(Category::Tv),
            "anime" => Some(Category::Anime),
            "documentary" => Some(Category::Documentary),
            "variety" => Some(Category::Variety),
            "sports" => Some(Category::Sports),
            "music" => Some(Category::Music),
            "ebook" => Some(Category::Ebook),
            "audiobook" => Some(Category::Audiobook),
            "software" => Some(Category::Software),
            "game" => Some(Category::Game),
            "course" => Some(Category::Course),
            "other" => Some(Category::Other),
            _ => None,
        }
    }

    /// Resolve from Chinese display name (e.g. "音乐" → Music).
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "电影" => Some(Category::Movie),
            "剧集" => Some(Category::Tv),
            "动漫" => Some(Category::Anime),
            "纪录片" => Some(Category::Documentary),
            "综艺" => Some(Category::Variety),
            "体育" => Some(Category::Sports),
            "音乐" => Some(Category::Music),
            "电子书" => Some(Category::Ebook),
            "有声书" => Some(Category::Audiobook),
            "软件" => Some(Category::Software),
            "游戏" => Some(Category::Game),
            "课程" => Some(Category::Course),
            "其他" => Some(Category::Other),
            _ => None,
        }
    }
}

/// Site-specific category mapping.
/// Maps (site_id, raw_category_id) → canonical Category.
pub fn map_site_category(site_id: &str, raw_category: &str) -> Option<Category> {
    match site_id {
        "m-team" => mteam_category(raw_category),
        "hdfans" => nexus_category(raw_category),
        "hdsky" => nexus_category(raw_category),
        "audiences" => nexus_category(raw_category),
        "azusa" => nexus_category(raw_category),
        "btschool" => nexus_category(raw_category),
        "chdbits" => nexus_category(raw_category),
        "hddolby" => nexus_category(raw_category),
        "HDHome" => nexus_category(raw_category),
        "hhan" => nexus_category(raw_category),
        "keepfrds" => nexus_category(raw_category),
        "ourbits" => nexus_category(raw_category),
        "pterclub" => nexus_category(raw_category),
        "ptsbao" => nexus_category(raw_category),
        "putao" => nexus_category(raw_category),
        "ssd" => nexus_category(raw_category),
        "ultrahd" => nexus_category(raw_category),
        "tjupt" => nexus_category(raw_category),
        "ttg" => nexus_category(raw_category),
        "hares" => nexus_category(raw_category),
        "hdatmos" => nexus_category(raw_category),
        "agsv" => nexus_category(raw_category),
        "filelist" => nexus_category(raw_category),
        "iptorrents" => nexus_category(raw_category),
        "exoticaz" => nexus_category(raw_category),
        "acgrip" => Some(Category::Anime),
        "mikanani" => Some(Category::Anime),
        "sukebei" => sukebei_category(raw_category),
        _ => None,
    }
}

/// M-Team category mapping (API returns numeric IDs like "400", "401", etc.)
fn mteam_category(raw: &str) -> Option<Category> {
    match raw {
        "400" | "401" | "419" => Some(Category::Movie),
        "402" => Some(Category::Tv),
        "403" | "405" | "433" | "440" => Some(Category::Anime),
        "404" | "441" => Some(Category::Documentary),
        "406" | "429" => Some(Category::Variety),
        "407" | "425" | "439" => Some(Category::Sports),
        "408" | "435" => Some(Category::Music),
        "430" | "434" | "443" => Some(Category::Music),
        "426" | "427" => Some(Category::Ebook),
        "428" | "437" | "442" => Some(Category::Audiobook),
        "423" | "431" => Some(Category::Software),
        "432" => Some(Category::Game),
        "420" | "438" => Some(Category::Course),
        "421" => Some(Category::Other),
        "409" | "436" => Some(Category::Other),
        _ => None,
    }
}

/// NexusPHP category mapping (standard across NexusPHP sites).
/// NexusPHP uses numeric category IDs like "401", "402", etc.
fn nexus_category(raw: &str) -> Option<Category> {
    match raw {
        // Movie
        "401" | "419" | "420" | "421" | "422" | "423" | "424" | "425" => Some(Category::Movie),
        // TV Series
        "402" | "426" | "427" | "428" | "429" => Some(Category::Tv),
        // Anime
        "403" | "430" | "431" | "432" | "433" => Some(Category::Anime),
        // Documentary
        "404" | "434" | "435" => Some(Category::Documentary),
        // Variety
        "405" | "436" | "437" => Some(Category::Variety),
        // Sports
        "406" | "438" | "439" | "440" => Some(Category::Sports),
        // Music
        "407" | "408" | "441" | "442" | "443" => Some(Category::Music),
        // Software
        "410" | "444" | "445" => Some(Category::Software),
        // Game
        "411" | "446" | "447" => Some(Category::Game),
        // Course
        "412" | "448" | "449" => Some(Category::Course),
        // Ebook
        "413" | "450" | "451" => Some(Category::Ebook),
        // Audiobook
        "414" | "452" | "453" => Some(Category::Audiobook),
        // Other
        "409" | "415" | "416" | "417" | "418" => Some(Category::Other),
        _ => None,
    }
}

/// Sukebei (adult) category mapping
fn sukebei_category(raw: &str) -> Option<Category> {
    match raw {
        "4" | "15" | "16" | "17" => Some(Category::Movie),
        "6" => Some(Category::Anime),
        "7" => Some(Category::Game),
        _ => Some(Category::Other),
    }
}

/// Get canonical ID string from category (for storage in DB as "1", "2", etc.)
pub fn canonical_id_from_category(cat: Category) -> String {
    cat.id().to_string()
}

/// Resolve category from a raw site category ID.
/// Returns (canonical_id_string, en_name, display_name).
pub fn resolve_category(site_id: &str, raw_category: &str) -> (String, String, String) {
    if let Some(cat) = map_site_category(site_id, raw_category) {
        (canonical_id_from_category(cat), cat.en_name().to_string(), cat.name().to_string())
    } else {
        // Unknown category - use "other"
        (
            canonical_id_from_category(Category::Other),
            Category::Other.en_name().to_string(),
            Category::Other.name().to_string(),
        )
    }
}

/// Resolve from English name, Chinese name, or canonical ID string.
pub fn resolve_from_str(s: &str) -> Option<Category> {
    // Try as integer ID first
    if let Ok(id) = s.parse::<i32>() {
        return Category::from_id(id);
    }
    // Try as English name
    Category::from_en_name(s)
        // Try as Chinese name
        .or_else(|| Category::from_name(s))
}

/// Normalize category string to English en_name for path lookup.
/// Accepts: English name ("music"), Chinese name ("音乐"), or canonical ID ("7").
pub fn category_to_en_name(s: &str) -> String {
    resolve_from_str(s)
        .map(|c| c.en_name().to_string())
        .unwrap_or_else(|| s.to_string())
}

/// Get all category names for display.
pub fn all_categories() -> Vec<(i32, &'static str, &'static str)> {
    vec![
        (Category::Movie.id(), Category::Movie.en_name(), Category::Movie.name()),
        (Category::Tv.id(), Category::Tv.en_name(), Category::Tv.name()),
        (Category::Anime.id(), Category::Anime.en_name(), Category::Anime.name()),
        (Category::Documentary.id(), Category::Documentary.en_name(), Category::Documentary.name()),
        (Category::Variety.id(), Category::Variety.en_name(), Category::Variety.name()),
        (Category::Sports.id(), Category::Sports.en_name(), Category::Sports.name()),
        (Category::Music.id(), Category::Music.en_name(), Category::Music.name()),
        (Category::Ebook.id(), Category::Ebook.en_name(), Category::Ebook.name()),
        (Category::Audiobook.id(), Category::Audiobook.en_name(), Category::Audiobook.name()),
        (Category::Software.id(), Category::Software.en_name(), Category::Software.name()),
        (Category::Game.id(), Category::Game.en_name(), Category::Game.name()),
        (Category::Course.id(), Category::Course.en_name(), Category::Course.name()),
        (Category::Other.id(), Category::Other.en_name(), Category::Other.name()),
    ]
}
