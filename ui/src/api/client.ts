/**
 * Typed API client for PT subscription management.
 *
 * All backend calls go through this module — no raw fetch elsewhere.
 */

const BASE = "/api/apps/pt-subscription";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const url = `${BASE}${path}`;
  const res = await fetch(url, {
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    ...init,
  });
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`${res.status}: ${text || res.statusText}`);
  }
  const json = (await res.json()) as { data?: T; ok?: boolean };
  return json.data ?? (json as unknown as T);
}

function get<T>(
  path: string,
  params?: Record<string, string | undefined>,
): Promise<T> {
  let url = path;
  if (params) {
    const qs = new URLSearchParams();
    for (const [k, v] of Object.entries(params)) {
      if (v !== undefined) qs.set(k, v);
    }
    const s = qs.toString();
    if (s) url += `?${s}`;
  }
  return request<T>(url);
}

function post<T>(path: string, body?: unknown): Promise<T> {
  return request<T>(path, {
    method: "POST",
    body: body ? JSON.stringify(body) : undefined,
  });
}

function put<T>(path: string, body: unknown): Promise<T> {
  return request<T>(path, { method: "PUT", body: JSON.stringify(body) });
}

function del<T>(path: string, body?: unknown): Promise<T> {
  return request<T>(path, {
    method: "DELETE",
    body: body ? JSON.stringify(body) : undefined,
  });
}

// ── Types ─────────────────────────────────────────────────────────────────────

export type DownloadClientType =
  | "qbittorrent"
  | "transmission"
  | "aria2"
  | "deluge"
  | "rtorrent"
  | "synology"
  | "xunlei"
  | "pan115";

export interface DownloadPath {
  type?: string;
  path: string;
  description: string;
}

export interface DownloadClientDto {
  id: string;
  name: string;
  type: string;
  url: string;
  username?: string;
  password?: string;
  isDefault: boolean;
  requireAuth: boolean;
  monitorEnabled: boolean;
  sortOrder: number;
  pollInterval: number;
  downloadPaths: DownloadPath[];
  createdAt: string;
  updatedAt: string;
}

export interface ClientStatusDto {
  id: string;
  name: string;
  type: string;
  isConnected: boolean;
  version?: string;
  errorMessage?: string;
}

export interface TorrentInfoDto {
  hash: string;
  name: string;
  size: number;
  progress: number;
  downloadSpeed: number;
  uploadSpeed: number;
  downloaded: number;
  uploaded: number;
  ratio: number;
  state: string;
  category?: string;
  tags?: string[];
  savePath: string;
  addedOn: number;
  completedOn?: number;
  seedingTime: number;
  eta?: number;
  numSeeds?: number;
  numLeeches?: number;
  tracker?: string;
}

export interface TransferInfoDto {
  dlSpeed: number;
  upSpeed: number;
  freeSpace: number;
}

export interface TorrentFileDto {
  index: number;
  name: string;
  size: number;
  progress: number;
  priority: number;
}

export interface PreviewFileItem {
  index: number;
  path: string;
  size: number;
  selected: boolean;
}

export interface DownloadFilterResult {
  totalFiles: number;
  excludedFiles: number;
  torrentName: string;
}

export interface CreateDownloadClientInput {
  name: string;
  type: string;
  url: string;
  username?: string;
  password?: string;
  isDefault?: boolean;
  requireAuth?: boolean;
  monitorEnabled?: boolean;
  pollInterval?: number;
  downloadPaths: DownloadPath[];
}

export interface UpdateDownloadClientInput {
  name?: string;
  type?: string;
  url?: string;
  username?: string | null;
  password?: string | null;
  isDefault?: boolean;
  requireAuth?: boolean;
  monitorEnabled?: boolean;
  pollInterval?: number;
  downloadPaths?: DownloadPath[];
}

export interface ReorderItem {
  id: string;
  sortOrder: number;
}

export interface AddTorrentBody {
  urls?: string[];
  torrents?: string[];
  savePath?: string;
  category?: string;
  tags?: string[];
  paused?: boolean;
  skipHashCheck?: boolean;
}

// ── API ───────────────────────────────────────────────────────────────────────

// ── Subscription types ────────────────────────────────────────────────────────

export interface Subscription {
  id: string;
  mediaType: "movie" | "tv";
  tmdbId: number | null;
  title: string;
  year: string | null;
  posterPath: string | null;
  season: number | null;
  episodes: number[] | null;
  category: string | null;
  sources: string[] | null;
  resolutions: string[] | null;
  codecs: string[] | null;
  releaseGroups: string[] | null;
  minSize: number;
  maxSize: number;
  minSeeders: number;
  maxSeeders: number;
  includeKeywords: string | null;
  excludeKeywords: string | null;
  freeOnly: boolean;
  excludeHr: boolean;
  maxDownloadsPerRun: number;
  intervalMinutes: number;
  siteIds: string[] | null;
  downloadClientId: string | null;
  status: "active" | "paused" | "pushed" | "completed" | "expired";
  lastCheckedAt: string | null;
  nextCheckAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateSubscriptionInput {
  mediaType: "movie" | "tv";
  tmdbId?: number;
  title: string;
  year?: string;
  posterPath?: string;
  season?: number;
  episodes?: number[];
  category?: string;
  sources?: string[];
  resolutions?: string[];
  codecs?: string[];
  releaseGroups?: string[];
  minSize?: number;
  maxSize?: number;
  minSeeders?: number;
  maxSeeders?: number;
  includeKeywords?: string;
  excludeKeywords?: string;
  freeOnly?: boolean;
  excludeHr?: boolean;
  maxDownloadsPerRun?: number;
  intervalMinutes?: number;
  siteIds?: string[];
  downloadClientId?: string;
}

// ── PT Site types ─────────────────────────────────────────────────────────────

export interface PtSiteDto {
  id: string;
  name: string;
  siteId: string;
  domain: string;
  authType: string;
  adultEnabled: boolean;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface PtSiteStatusDto {
  id: string;
  name: string;
  isLoggedIn: boolean;
  userInfo: PtUserInfoDto | null;
  lastCheckedAt: string | null;
  errorMessage: string | null;
}

export interface PtUserInfoDto {
  uid: string;
  username: string;
  uploaded: string;
  downloaded: string;
  shareRatio: string;
  seeding: number;
  leeching: number;
  bonus: string;
}

export interface AvailableSiteDto {
  id: string;
  name: string;
  domain: string;
  allowAuthType: string[];
  hasAdultContent: boolean;
  adultOnly: boolean;
}

// ── Subscription API ──────────────────────────────────────────────────────────

export const subscriptionsApi = {
  list: () => get<Subscription[]>("/subscriptions"),
  getById: (id: string) => get<Subscription>(`/subscriptions/${id}`),
  create: (input: CreateSubscriptionInput) =>
    post<Subscription>("/subscriptions", input),
  update: (id: string, input: Partial<CreateSubscriptionInput>) =>
    put<Subscription>(`/subscriptions/${id}`, input),
  delete: (id: string) => del(`/subscriptions/${id}`),
  execute: (id: string) => post(`/subscriptions/${id}/execute`),
  getLogs: (id: string, limit?: number) =>
    get<SubscriptionLogEntry[]>(
      `/subscriptions/${id}/logs`,
      limit ? { limit: String(limit) } : undefined,
    ),
  getRawLogs: (id: string) =>
    fetch(`/api/apps/pt-subscription/subscriptions/${id}/logs/raw`).then((r) =>
      r.text(),
    ),
  getRunLogs: (id: string, runId: string) =>
    get<SubscriptionLogEntry[]>(`/subscriptions/${id}/runs/${runId}/logs`),
  getDebugInfo: (id: string) =>
    get<SubscriptionDebugInfo>(`/subscriptions/${id}/debug`),
};

export interface SubscriptionLogEntry {
  timestamp: string;
  subscriptionId: string;
  runId: string;
  phase: string;
  message: string;
  details?: Record<string, unknown>;
}

export interface SubscriptionRunSummary {
  runId: string;
  startedAt: string;
  completedAt?: string;
  totalFound: number;
  afterFilter: number;
  matched: boolean;
  matchedTorrent?: string;
  downloaded: boolean;
  isRunning: boolean;
  error?: string;
}

export interface SubscriptionDebugInfo {
  subscription: Subscription;
  totalRuns: number;
  successfulDownloads: number;
  lastMatchedAt?: string;
  activeRunId?: string;
  recentRuns: SubscriptionRunSummary[];
}

// ── PT Sites API ──────────────────────────────────────────────────────────────

export const ptSitesApi = {
  list: () => get<PtSiteDto[]>("/subscriptions/pt-sites"),
  listWithStatus: () =>
    get<PtSiteStatusDto[]>("/subscriptions/pt-sites/with-status"),
  allStatus: () => get<PtSiteStatusDto[]>("/subscriptions/pt-sites/all-status"),
  getAvailableSites: () =>
    get<AvailableSiteDto[]>("/subscriptions/pt-sites/available"),
  getById: (id: string) => get<PtSiteDto>(`/subscriptions/pt-sites/${id}`),
  create: (input: Partial<PtSiteDto>) =>
    post<PtSiteDto>("/subscriptions/pt-sites", input),
  update: (id: string, input: Partial<PtSiteDto>) =>
    put<PtSiteDto>(`/subscriptions/pt-sites/${id}`, input),
  delete: (id: string) => del(`/subscriptions/pt-sites/${id}`),
  testConnection: (id: string) =>
    get<PtSiteStatusDto>(`/subscriptions/pt-sites/${id}/status`),
};

// ── Filter Options API ─────────────────────────────────────────────────────

export interface FilterOptions {
  sources: string[];
  resolutions: string[];
  codecs: string[];
}

export const filterOptionsApi = {
  list: () => get<FilterOptions>("/subscriptions/filter-options"),
};

// ── Search API ────────────────────────────────────────────────────────────────

export interface PtSearchResultWithSite {
  id: string;
  title: string;
  subtitle?: string;
  size: string;
  sizeBytes?: number;
  seeders: number;
  leechers: number;
  grabs?: number;
  category: string;
  categoryName: string;
  categoryDisplayName: string;
  uploadTime: string;
  downloadUrl: string;
  detailUrl: string;
  posterUrl?: string;
  imdbUrl?: string;
  imdbRating?: string;
  doubanUrl?: string;
  doubanRating?: string;
  discount?: string;
  discountEndTime?: string;
  videoCodec?: string;
  audioCodec?: string;
  resolution?: string;
  source?: string;
  downloadVolumeFactor?: number;
  uploadVolumeFactor?: number;
  siteDbId: string;
  siteName: string;
}

export interface SiteSummary {
  siteDbId: string;
  siteName: string;
  count: number;
}

export interface PtSearchResponse {
  results: PtSearchResultWithSite[];
  siteSummaries: SiteSummary[];
  total: number;
}

export const searchApi = {
  searchPt: (keyword: string, siteIds?: string[], categories?: string[]) =>
    post<PtSearchResponse>("/subscriptions/search/pt", {
      keyword,
      siteIds: siteIds ?? [],
      categories: categories ?? [],
    }),
};

// ── Categories API ───────────────────────────────────────────────────────

export interface CategoryDto {
  id: number;
  name: string;
  enName: string;
}

export interface CategoriesResponse {
  categories: CategoryDto[];
}

export const categoriesApi = {
  list: () => get<CategoriesResponse>("/subscriptions/categories"),
};

// ── Downloads API ─────────────────────────────────────────────────────────────

export const downloadsApi = {
  clients: {
    list: () => get<DownloadClientDto[]>("/clients"),
    getById: (id: string) => get<DownloadClientDto>(`/clients/${id}`),
    create: (input: CreateDownloadClientInput) =>
      post<DownloadClientDto>("/clients", input),
    update: (id: string, input: UpdateDownloadClientInput) =>
      put<DownloadClientDto>(`/clients/${id}`, input),
    delete: (id: string) => del(`/clients/${id}`),
    setDefault: (id: string) => post(`/clients/${id}/set-default`),
    reorder: (items: ReorderItem[]) => post("/clients/reorder", items),
    testConnection: (id: string) =>
      get<ClientStatusDto>(`/clients/${id}/test-connection`),
    allStatus: () => get<ClientStatusDto[]>("/clients/all-status"),
  },
  torrents: {
    list: (clientId: string, params?: { filter?: string; category?: string }) =>
      get<TorrentInfoDto[]>(
        `/clients/${clientId}/torrents`,
        params as Record<string, string | undefined>,
      ),
    add: (clientId: string, body: AddTorrentBody) =>
      post(`/clients/${clientId}/torrents`, body),
    pause: (clientId: string, hashes: string[]) =>
      post(`/clients/${clientId}/torrents/pause`, { hashes }),
    resume: (clientId: string, hashes: string[]) =>
      post(`/clients/${clientId}/torrents/resume`, { hashes }),
    delete: (clientId: string, hashes: string[], deleteFiles?: boolean) =>
      del(`/clients/${clientId}/torrents`, { hashes, deleteFiles }),
    transferInfo: (clientId: string) =>
      get<TransferInfoDto>(`/clients/${clientId}/transfer-info`),
    files: (clientId: string, hash: string) =>
      get<TorrentFileDto[]>(`/clients/${clientId}/torrent-files/${hash}`),
  },

  // Torrent preview and filtered download
  torrent: {
    preview: (body: { siteId: string; torrentId: string }) =>
      post<PreviewFileItem[]>("/torrent/preview", body),
    resolvePath: (body: { clientId: string; category?: string }) =>
      post<{ path: string | null; allPaths: unknown[] }>(
        "/torrent/resolve-path",
        body,
      ),
    downloadFiltered: (body: {
      clientId: string;
      siteId: string;
      torrentId: string;
      savePath?: string;
      category?: string;
      category?: string;
      tags?: string[];
      siteId?: string;
      apiKey?: string;
      cookies?: string;
      season?: number;
      episodes?: number[];
    }) => post<DownloadFilterResult>("/torrent/download-filtered", body),
  },
};
