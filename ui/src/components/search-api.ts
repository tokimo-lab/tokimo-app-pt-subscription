// ── TMDB types ───────────────────────────────────────────────────────────

export interface TmdbMedia {
  id: number;
  mediaType: string;
  title: string;
  originalTitle?: string;
  overview?: string;
  posterPath?: string;
  backdropPath?: string;
  releaseDate?: string;
  voteAverage?: number;
  voteCount?: number;
  popularity?: number;
  genreIds?: number[];
  imdbId?: string;
  source?: string;
  imdbRating?: number;
  totalSeasons?: number;
}

export interface TmdbSearchResult {
  keyword: string;
  tmdbResults: TmdbMedia[];
  timeTaken: number;
}

// ── Search history types ─────────────────────────────────────────────────

export interface SearchHistoryItem {
  id: string;
  keyword: string;
  searchedAt: string;
}

// ── API calls ────────────────────────────────────────────────────────────

export async function fetchTmdbSearch(
  keyword: string,
): Promise<TmdbSearchResult> {
  const resp = await fetch("/api/apps/search", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ keyword }),
  });
  if (!resp.ok) throw new Error(`${resp.status}`);
  const json = await resp.json();
  return json.data ?? json;
}

export async function fetchSearchHistory(): Promise<SearchHistoryItem[]> {
  const resp = await fetch("/api/apps/search/history", {
    credentials: "include",
  });
  if (!resp.ok) return [];
  const json = await resp.json();
  return json.data ?? [];
}

export async function addSearchHistory(
  keyword: string,
): Promise<SearchHistoryItem[]> {
  const resp = await fetch("/api/apps/search/history", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    credentials: "include",
    body: JSON.stringify({ keyword }),
  });
  if (!resp.ok) return [];
  const json = await resp.json();
  return json.data ?? [];
}

export async function removeSearchHistory(
  keyword: string,
): Promise<SearchHistoryItem[]> {
  const resp = await fetch("/api/apps/search/history", {
    method: "DELETE",
    headers: { "Content-Type": "application/json" },
    credentials: "include",
    body: JSON.stringify({ keyword }),
  });
  if (!resp.ok) return [];
  const json = await resp.json();
  return json.data ?? [];
}

export async function clearSearchHistory(): Promise<SearchHistoryItem[]> {
  const resp = await fetch("/api/apps/search/history/all", {
    method: "DELETE",
    credentials: "include",
  });
  if (!resp.ok) return [];
  return [];
}
