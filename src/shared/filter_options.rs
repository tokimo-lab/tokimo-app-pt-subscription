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
