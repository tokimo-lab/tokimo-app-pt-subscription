import { Button, cn, Progress, Tag } from "@tokimo/ui";
import { Pause, Play, RefreshCw, Search, Trash2 } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  type ClientStatusDto,
  type DownloadClientDto,
  downloadsApi,
  type TorrentInfoDto,
  type TransferInfoDto,
} from "../api/client";

const STATE_COLORS: Record<string, string> = {
  downloading: "blue",
  uploading: "green",
  seeding: "green",
  pausedDL: "default",
  pausedUP: "default",
  queuedDL: "orange",
  queuedUP: "orange",
  stalledDL: "orange",
  stalledUP: "orange",
  checkingDL: "cyan",
  checkingUP: "cyan",
  error: "red",
  missingFiles: "red",
  unknown: "default",
};

function fmtBytes(b: number): string {
  if (b >= 1_073_741_824) return `${(b / 1_073_741_824).toFixed(1)} GB`;
  if (b >= 1_048_576) return `${(b / 1_048_576).toFixed(1)} MB`;
  if (b >= 1024) return `${(b / 1024).toFixed(1)} KB`;
  return `${b} B`;
}

function fmtSpeed(b: number): string {
  return `${fmtBytes(b)}/s`;
}

function fmtEta(seconds: number | null | undefined): string {
  if (!seconds || seconds <= 0) return "—";
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  return `${h}h${m}m`;
}

interface Props {
  client: DownloadClientDto;
  status: ClientStatusDto | null;
  toast: { success: (msg: string) => void; error: (msg: string) => void };
}

export function TorrentListPanel({ client, status, toast }: Props) {
  const [torrents, setTorrents] = useState<TorrentInfoDto[]>([]);
  const [transfer, setTransfer] = useState<TransferInfoDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [stateFilter, _setStateFilter] = useState<string | null>(null);
  const eventSourceRef = useRef<EventSource | null>(null);

  const fetchTorrents = useCallback(async () => {
    try {
      const [t, info] = await Promise.all([
        downloadsApi.torrents.list(client.id),
        downloadsApi.torrents.transferInfo(client.id),
      ]);
      setTorrents(t);
      setTransfer(info);
    } catch (e) {
      // Only show error if not SSE-managed
      console.error("Failed to fetch torrents:", e);
    } finally {
      setLoading(false);
    }
  }, [client.id]);

  // SSE for real-time updates
  useEffect(() => {
    const url = `/api/apps/downloads/clients/${client.id}/torrent-events`;
    const es = new EventSource(url);
    eventSourceRef.current = es;

    es.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as {
          torrents: TorrentInfoDto[];
          transfer?: TransferInfoDto;
          error?: string;
        };
        if (!data.error) {
          setTorrents(data.torrents);
          if (data.transfer) setTransfer(data.transfer);
          setLoading(false);
        }
      } catch {
        // ignore parse errors
      }
    };

    es.onerror = () => {
      // Fallback to polling on SSE error
      void fetchTorrents();
    };

    return () => {
      es.close();
      eventSourceRef.current = null;
    };
  }, [client.id, fetchTorrents]);

  // Initial fetch
  useEffect(() => {
    void fetchTorrents();
  }, [fetchTorrents]);

  // Filter torrents
  const filtered = torrents.filter((t) => {
    if (search && !t.name.toLowerCase().includes(search.toLowerCase()))
      return false;
    if (stateFilter && t.state !== stateFilter) return false;
    return true;
  });

  const handlePause = async () => {
    const hashes = [...selected];
    if (hashes.length === 0) return;
    try {
      await downloadsApi.torrents.pause(client.id, hashes);
      toast.success(`已暂停 ${hashes.length} 个种子`);
      setSelected(new Set());
      void fetchTorrents();
    } catch (e) {
      toast.error(`暂停失败: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleResume = async () => {
    const hashes = [...selected];
    if (hashes.length === 0) return;
    try {
      await downloadsApi.torrents.resume(client.id, hashes);
      toast.success(`已恢复 ${hashes.length} 个种子`);
      setSelected(new Set());
      void fetchTorrents();
    } catch (e) {
      toast.error(`恢复失败: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleDelete = async () => {
    const hashes = [...selected];
    if (hashes.length === 0) return;
    if (!confirm(`确定删除 ${hashes.length} 个种子？`)) return;
    try {
      await downloadsApi.torrents.delete(client.id, hashes, false);
      toast.success(`已删除 ${hashes.length} 个种子`);
      setSelected(new Set());
      void fetchTorrents();
    } catch (e) {
      toast.error(`删除失败: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const toggleSelect = (hash: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(hash)) next.delete(hash);
      else next.add(hash);
      return next;
    });
  };

  const toggleSelectAll = () => {
    if (selected.size === filtered.length) {
      setSelected(new Set());
    } else {
      setSelected(new Set(filtered.map((t) => t.hash)));
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header with stats */}
      <div className="p-3 border-b border-border">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            <h2 className="font-semibold">{client.name}</h2>
            <Tag color={status?.isConnected ? "green" : "red"}>
              {status?.isConnected ? "已连接" : "未连接"}
            </Tag>
          </div>
          <Button
            variant="text"
            size="small"
            onClick={() => void fetchTorrents()}
            cursor-pointer
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>

        {/* Transfer stats */}
        {transfer && (
          <div className="flex gap-4 text-sm text-muted-foreground">
            <span>↓ {fmtSpeed(transfer.dlSpeed)}</span>
            <span>↑ {fmtSpeed(transfer.upSpeed)}</span>
            <span>剩余: {fmtBytes(transfer.freeSpace)}</span>
            <span>共 {torrents.length} 个种子</span>
          </div>
        )}
      </div>

      {/* Toolbar */}
      <div className="p-2 border-b border-border flex items-center gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-2 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <input
            className="w-full pl-8 pr-3 py-1.5 text-sm border border-border rounded-md bg-background"
            placeholder="搜索种子..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>

        {selected.size > 0 && (
          <>
            <Button
              variant="default"
              size="small"
              onClick={() => void handlePause()}
              cursor-pointer
            >
              <Pause className="h-3.5 w-3.5 mr-1" />
              暂停 ({selected.size})
            </Button>
            <Button
              variant="default"
              size="small"
              onClick={() => void handleResume()}
              cursor-pointer
            >
              <Play className="h-3.5 w-3.5 mr-1" />
              恢复
            </Button>
            <Button
              variant="default"
              size="small"
              onClick={() => void handleDelete()}
              className="text-destructive"
              cursor-pointer
            >
              <Trash2 className="h-3.5 w-3.5 mr-1" />
              删除
            </Button>
          </>
        )}
      </div>

      {/* Torrent list */}
      <div className="flex-1 overflow-y-auto">
        {loading ? (
          <div className="flex items-center justify-center h-full text-muted-foreground">
            加载中...
          </div>
        ) : filtered.length === 0 ? (
          <div className="flex items-center justify-center h-full text-muted-foreground">
            {search ? "没有匹配的种子" : "暂无种子任务"}
          </div>
        ) : (
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-background border-b border-border">
              <tr className="text-left text-muted-foreground">
                <th className="p-2 w-8">
                  <input
                    type="checkbox"
                    checked={
                      selected.size === filtered.length && filtered.length > 0
                    }
                    onChange={toggleSelectAll}
                    className="cursor-pointer"
                  />
                </th>
                <th className="p-2">名称</th>
                <th className="p-2 w-20">大小</th>
                <th className="p-2 w-32">进度</th>
                <th className="p-2 w-24">↓速度</th>
                <th className="p-2 w-24">↑速度</th>
                <th className="p-2 w-16">ETA</th>
                <th className="p-2 w-24">状态</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((t) => (
                <tr
                  key={t.hash}
                  className={cn(
                    "border-b border-border/50 hover:bg-accent/30 cursor-pointer",
                    selected.has(t.hash) && "bg-accent/50",
                  )}
                  onClick={() => toggleSelect(t.hash)}
                >
                  <td className="p-2">
                    <input
                      type="checkbox"
                      checked={selected.has(t.hash)}
                      onChange={() => toggleSelect(t.hash)}
                      onClick={(e) => e.stopPropagation()}
                      className="cursor-pointer"
                    />
                  </td>
                  <td className="p-2 max-w-xs truncate" title={t.name}>
                    {t.name}
                  </td>
                  <td className="p-2 text-muted-foreground">
                    {fmtBytes(t.size)}
                  </td>
                  <td className="p-2">
                    <div className="flex items-center gap-2">
                      <Progress
                        percent={t.progress * 100}
                        className="h-1.5 flex-1"
                      />
                      <span className="text-xs text-muted-foreground w-10 text-right">
                        {(t.progress * 100).toFixed(1)}%
                      </span>
                    </div>
                  </td>
                  <td className="p-2 text-muted-foreground">
                    {fmtSpeed(t.downloadSpeed)}
                  </td>
                  <td className="p-2 text-muted-foreground">
                    {fmtSpeed(t.uploadSpeed)}
                  </td>
                  <td className="p-2 text-muted-foreground">{fmtEta(t.eta)}</td>
                  <td className="p-2">
                    <Tag color={STATE_COLORS[t.state] ?? "default"}>
                      {t.state}
                    </Tag>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
