/// Type of PT site backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteType {
    NexusPhp,
    Api,
    Public,
}

/// Top-level site configuration
#[derive(Debug, Clone)]
pub struct SiteConfig {
    pub site_id: &'static str,
    pub name: &'static str,
    pub site_type: SiteType,
    pub search: SearchConfig,
    pub adult_search: Option<SearchConfig>,
    pub row_selector: &'static str,
    pub fields: FieldMap,
}

/// Search configuration – either HTML or API
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum SearchConfig {
    Html(HtmlSearchConfig),
    Api(ApiSearchConfig),
}

/// HTML-based search (`NexusPhp` / Public sites)
#[derive(Debug, Clone)]
pub struct HtmlSearchConfig {
    pub path: &'static str,
    pub query: Vec<(&'static str, &'static str)>,
}

/// API-based search (M-Team)
#[derive(Debug, Clone)]
pub struct ApiSearchConfig {
    pub path: &'static str,
    pub method: &'static str,
    pub headers: Vec<(&'static str, &'static str)>,
    pub body_template: &'static str,
    pub data_path: Option<&'static str>,
    pub list_path: Option<&'static str>,
    pub fields: ApiFieldMap,
}

/// JSON field path mapping for API sites
#[derive(Debug, Clone)]
pub struct ApiFieldMap {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: Option<&'static str>,
    pub size: &'static str,
    pub seeders: &'static str,
    pub leechers: &'static str,
    pub grabs: Option<&'static str>,
    pub category: Option<&'static str>,
    pub upload_time: Option<&'static str>,
    pub detail_url: Option<&'static str>,
    pub poster_url: Option<&'static str>,
    pub imdb_url: Option<&'static str>,
    pub imdb_rating: Option<&'static str>,
    pub douban_url: Option<&'static str>,
    pub douban_rating: Option<&'static str>,
    pub discount: Option<&'static str>,
    pub discount_end_time: Option<&'static str>,
    pub video_codec: Option<&'static str>,
    pub audio_codec: Option<&'static str>,
    pub resolution: Option<&'static str>,
    pub source: Option<&'static str>,
}

/// A single field extraction rule for HTML parsing
#[derive(Debug, Clone, Default)]
pub struct FieldConfig {
    pub selector: Option<&'static str>,
    pub attribute: Option<&'static str>,
    pub default_value: Option<&'static str>,
    pub filters: Vec<Filter>,
    /// Case-matching: check selectors in order, return value of first match.
    /// `("*", "1")` is the wildcard/default.
    pub case: Vec<(&'static str, &'static str)>,
    /// Template referencing other extracted fields
    pub text: Option<&'static str>,
    pub method: Option<&'static str>,
}

impl FieldConfig {
    pub const fn selector(sel: &'static str) -> Self {
        Self {
            selector: Some(sel),
            attribute: None,
            default_value: None,
            filters: Vec::new(),
            case: Vec::new(),
            text: None,
            method: None,
        }
    }

    pub const fn text_tpl(tpl: &'static str) -> Self {
        Self {
            selector: None,
            attribute: None,
            default_value: None,
            filters: Vec::new(),
            case: Vec::new(),
            text: Some(tpl),
            method: None,
        }
    }

    pub const fn default_val(val: &'static str) -> Self {
        Self {
            selector: None,
            attribute: None,
            default_value: Some(val),
            filters: Vec::new(),
            case: Vec::new(),
            text: None,
            method: None,
        }
    }
}

/// Filter operations applied after field extraction
#[derive(Debug, Clone)]
pub enum Filter {
    /// Regex search: extract first match (pattern, `group_index`)
    ReSearch(&'static str, usize),
    /// String replace (search, replacement)
    Replace(&'static str, &'static str),
    /// Extract query-string parameter
    QueryString(&'static str),
    /// Split by separator, take index
    Split(&'static str, usize),
    /// Date parse (ignored for now, just passes through)
    DateParse(&'static str),
}

/// All fields for torrent row extraction
#[derive(Debug, Clone)]
pub struct FieldMap {
    pub id: FieldConfig,
    pub title_default: FieldConfig,
    pub title_optional: FieldConfig,
    pub title: FieldConfig,
    pub description: Option<FieldConfig>,
    pub category: Option<FieldConfig>,
    pub details: Option<FieldConfig>,
    pub download: FieldConfig,
    pub date_elapsed: Option<FieldConfig>,
    pub date_added: Option<FieldConfig>,
    pub date: Option<FieldConfig>,
    pub size: FieldConfig,
    pub seeders: FieldConfig,
    pub leechers: FieldConfig,
    pub grabs: Option<FieldConfig>,
    pub downloadvolumefactor: FieldConfig,
    pub uploadvolumefactor: FieldConfig,
    pub free_deadline: Option<FieldConfig>,
    pub imdbid: Option<FieldConfig>,
    /// Extra fields needed for template resolution (tags, subject, etc.)
    pub extra: Vec<(&'static str, FieldConfig)>,
}

/// Map (downloadvolumefactor, uploadvolumefactor) → human-readable discount string
pub fn factors_to_discount(dl_factor: f64, ul_factor: f64) -> Option<String> {
    if (dl_factor - 0.0).abs() < 0.01 && (ul_factor - 2.0).abs() < 0.01 {
        Some("2X_FREE".into())
    } else if (dl_factor - 0.0).abs() < 0.01 {
        Some("FREE".into())
    } else if (dl_factor - 0.5).abs() < 0.01 && (ul_factor - 2.0).abs() < 0.01 {
        Some("2X_PERCENT_50".into())
    } else if (dl_factor - 0.5).abs() < 0.01 {
        Some("PERCENT_50".into())
    } else if (dl_factor - 0.3).abs() < 0.01 {
        Some("PERCENT_30".into())
    } else if (ul_factor - 2.0).abs() < 0.01 {
        Some("2X".into())
    } else {
        None
    }
}

/// Format bytes into human-readable string
pub fn format_size(value: &str) -> String {
    // If already has unit, return as-is
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "0" {
        return "0 B".into();
    }

    // Check if it already looks like "1.23 GB"
    let re = regex::Regex::new(r"(?i)^\d+(\.\d+)?\s*(B|KB|MB|GB|TB|PB)$").unwrap();
    if re.is_match(trimmed) {
        return trimmed.to_string();
    }

    // Try parse as number (bytes)
    if let Ok(bytes) = trimmed.parse::<f64>() {
        return bytes_to_string(bytes);
    }

    trimmed.to_string()
}

fn bytes_to_string(bytes: f64) -> String {
    if bytes <= 0.0 {
        return "0 B".into();
    }
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let i = (bytes.ln() / 1024_f64.ln()).floor() as usize;
    let i = i.min(UNITS.len() - 1);
    let value = bytes / 1024_f64.powi(i as i32);
    format!("{:.2} {}", value, UNITS[i])
}
