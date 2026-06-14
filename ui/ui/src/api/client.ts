/**
 * Typed API client for download tool management.
 *
 * All backend calls go through this module — no raw fetch elsewhere.
 */

const BASE = "/api/apps/download-clients";

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
};
