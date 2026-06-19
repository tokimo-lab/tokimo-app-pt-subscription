import type { TFunction } from "i18next";
import type { PtSearchResultWithSite } from "../api/client";

export const PT_PAGE_SIZE = 20;

// ── Title tag parsing ────────────────────────────────────────────────────

export interface TitleTags {
  resolution?: string;
  videoCodec?: string;
  audioCodec?: string;
  source?: string;
}

/** Parse resolution, video/audio codec, source from torrent title. */
export function parseTitleTags(title: string, subtitle?: string): TitleTags {
  const upper = `${title} ${subtitle ?? ""}`.toUpperCase();
  const tags: TitleTags = {};

  // Resolution
  if (upper.includes("2160P") || upper.includes("4K"))
    tags.resolution = "2160p";
  else if (upper.includes("1080P")) tags.resolution = "1080p";
  else if (upper.includes("720P")) tags.resolution = "720p";
  else if (upper.includes("480P") || upper.includes("576P"))
    tags.resolution = "480p";

  // Video codec
  if (
    upper.includes("H265") ||
    upper.includes("H.265") ||
    upper.includes("HEVC")
  )
    tags.videoCodec = "H265";
  else if (upper.includes("X265")) tags.videoCodec = "x265";
  else if (
    upper.includes("H264") ||
    upper.includes("H.264") ||
    upper.includes("AVC")
  )
    tags.videoCodec = "H264";
  else if (upper.includes("X264")) tags.videoCodec = "x264";
  else if (upper.includes("AV1")) tags.videoCodec = "AV1";
  else if (upper.includes("VC-1") || upper.includes("VC1"))
    tags.videoCodec = "VC-1";

  // Audio codec
  if (upper.includes("ATMOS")) tags.audioCodec = "Atmos";
  else if (upper.includes("DTS-HD") || upper.includes("DTSHD"))
    tags.audioCodec = "DTS-HD";
  else if (upper.includes("TRUEHD") || upper.includes("TRUE-HD"))
    tags.audioCodec = "TrueHD";
  else if (upper.includes("DTS")) tags.audioCodec = "DTS";
  else if (upper.includes("FLAC")) tags.audioCodec = "FLAC";
  else if (upper.includes("AAC")) tags.audioCodec = "AAC";
  else if (
    upper.includes("AC3") ||
    upper.includes("AC-3") ||
    upper.match(/\bDD\b/)
  )
    tags.audioCodec = "AC3";
  else if (upper.includes("OPUS")) tags.audioCodec = "Opus";

  // Source
  if (upper.includes("REMUX")) tags.source = "Remux";
  else if (upper.includes("BLURAY") || upper.includes("BLU-RAY"))
    tags.source = "BluRay";
  else if (upper.includes("WEB-DL") || upper.includes("WEBDL"))
    tags.source = "WEB-DL";
  else if (upper.includes("WEBRIP")) tags.source = "WEBRip";
  else if (upper.includes("HDTV")) tags.source = "HDTV";
  else if (upper.includes("DVDRIP") || upper.includes("DVD"))
    tags.source = "DVDRip";

  return tags;
}

// ── Resolution normalization ──────────────────────────────────────────────

const normalizeResolution = (res: string): string => {
  const lower = res.toLowerCase();
  if (lower.includes("4k") || lower.includes("2160")) return "4K/2160p";
  if (lower.includes("1080")) return "1080p";
  if (lower.includes("720")) return "720p";
  if (lower.includes("sd") || lower.includes("480") || lower.includes("576"))
    return "SD";
  return res;
};

const SEASON_EPISODE_RE =
  /S(\d{1,4})(?:\s*E(\d{1,4})(?:\s*-\s*E?(\d{1,4}))?)?/gi;

const YEAR_RE = /(?:^|[\s.([])((?:19|20)\d{2})(?:[\s.)\]]|$)/g;

const parseYear = (text: string): string | null => {
  const match = YEAR_RE.exec(text);
  YEAR_RE.lastIndex = 0;
  return match ? match[1] : null;
};

export interface DynamicFilterOptions {
  resolutions: string[];
  videoCodecs: string[];
  discounts: string[];
  years: string[];
  seasons: string[];
  episodes: string[];
  hasEpisodes: boolean;
}

export const extractFilterOptions = (
  torrents: PtSearchResultWithSite[],
): DynamicFilterOptions => {
  const resSet = new Set<string>();
  const codecSet = new Set<string>();
  const discountSet = new Set<string>();
  const yearSet = new Set<string>();
  const seasonSet = new Set<string>();
  const episodeSet = new Set<string>();

  for (const t of torrents) {
    // Use title-parsed values for filters (API may return category IDs)
    const titleTags = parseTitleTags(t.title, t.subtitle);
    if (titleTags.resolution)
      resSet.add(normalizeResolution(titleTags.resolution));
    if (titleTags.videoCodec) codecSet.add(titleTags.videoCodec);
    const dlFactor = getDownloadFactor(t);
    const ulFactor = getUploadFactor(t);
    if (dlFactor === 0) discountSet.add("free");
    if (dlFactor === 0.5) discountSet.add("half");
    if (ulFactor >= 2) discountSet.add("2x");

    const fullText = `${t.title} ${t.subtitle ?? ""}`;
    const year = parseYear(fullText);
    if (year) yearSet.add(year);

    let match: RegExpExecArray | null;
    SEASON_EPISODE_RE.lastIndex = 0;
    // biome-ignore lint/suspicious/noAssignInExpressions: regex exec loop
    while ((match = SEASON_EPISODE_RE.exec(fullText)) !== null) {
      seasonSet.add(match[1]);
      if (match[2]) {
        const start = Number.parseInt(match[2], 10);
        const end = match[3] ? Number.parseInt(match[3], 10) : start;
        for (let e = start; e <= end; e++) episodeSet.add(String(e));
      }
    }
  }

  const sortNumeric = (a: string, b: string) => Number(a) - Number(b);

  return {
    resolutions: [...resSet].sort(),
    videoCodecs: [...codecSet].sort(),
    discounts: [...discountSet],
    years: [...yearSet].sort().reverse(),
    seasons: [...seasonSet].sort(sortNumeric),
    episodes: [...episodeSet].sort(sortNumeric),
    hasEpisodes: episodeSet.size > 0 || seasonSet.size > 0,
  };
};

export const getDiscountLabel = (value: string): string => {
  switch (value) {
    case "free":
      return "免费";
    case "half":
      return "50%";
    case "2x":
      return "2x↑";
    default:
      return value;
  }
};

export const formatDate = (dateStr: string | undefined): string => {
  if (!dateStr) return "";
  try {
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    if (days === 0) return "今天";
    if (days === 1) return "昨天";
    if (days < 7) return `${days}天前`;
    if (days < 30) return `${Math.floor(days / 7)}周前`;
    if (days < 365) return `${Math.floor(days / 30)}月前`;
    return `${Math.floor(days / 365)}年前`;
  } catch {
    return dateStr;
  }
};

// ── Category slugs ────────────────────────────────────────────────────────

/** Translate a canonical category slug, falling back to the raw slug. */
export function categoryLabel(slug: string, t: TFunction): string {
  return t(`category.${slug}`, { defaultValue: slug });
}

// Canonical category slug → tag color
const CATEGORY_COLORS: Record<string, string> = {
  movie: "blue",
  tv: "green",
  anime: "purple",
  documentary: "cyan",
  variety: "orange",
  sports: "red",
  music: "pink",
  ebook: "gold",
  audiobook: "gold",
};

export function getCategoryColor(slug: string): string {
  return CATEGORY_COLORS[slug] ?? "default";
}

export function getDownloadFactor(torrent: PtSearchResultWithSite): number {
  if (torrent.downloadVolumeFactor !== undefined)
    return torrent.downloadVolumeFactor;
  const d = torrent.discount?.toUpperCase();
  if (d === "FREE") return 0;
  if (d === "PERCENT_50") return 0.5;
  if (d === "PERCENT_70") return 0.3;
  return 1;
}

export function getUploadFactor(torrent: PtSearchResultWithSite): number {
  if (torrent.uploadVolumeFactor !== undefined)
    return torrent.uploadVolumeFactor;
  const d = torrent.discount?.toUpperCase();
  if (d === "2X_FREE" || d === "2X") return 2;
  return 1;
}
