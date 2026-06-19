/**
 * 下载确认弹窗 — 复制自主 web app 的 DownloadConfirmModal，适配 pt-subscription app
 */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Button,
  Checkbox,
  Collapse,
  Form,
  Select,
  Spin,
  Tag,
  Tooltip,
} from "@tokimo/ui";
import { ArrowDown, ArrowUp, ChevronsUpDown } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import type { PreviewFileItem, PtSearchResultWithSite } from "../api/client";
import { downloadsApi } from "../api/client";
import { TorrentCardBody } from "./TorrentCard";

interface WindowHandle {
  id: string;
  metadata: Record<string, unknown>;
  close: () => void;
}

interface Props {
  win: WindowHandle;
}

const _ns = "media.downloadConfirm";

const formatSize = (bytes: number): string => {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / 1024 ** i;
  return `${val.toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
};

type FileCategory =
  | "video"
  | "audio"
  | "image"
  | "subtitle"
  | "ebook"
  | "document"
  | "archive"
  | "other";

const fileTypeColor: Record<FileCategory, string> = {
  video: "green",
  audio: "orange",
  image: "purple",
  subtitle: "blue",
  ebook: "cyan",
  document: "gold",
  archive: "red",
  other: "default",
};

const fileTypeLabel: Record<FileCategory, string> = {
  video: "视频",
  audio: "音频",
  image: "图片",
  subtitle: "字幕",
  ebook: "电子书",
  document: "文档",
  archive: "压缩包",
  other: "其他",
};

const VIDEO_EXTS = new Set([
  "mkv",
  "mp4",
  "avi",
  "wmv",
  "ts",
  "m2ts",
  "flv",
  "mov",
  "rmvb",
  "webm",
  "mpg",
  "mpeg",
  "vob",
  "iso",
  "m4v",
  "f4v",
  "ogv",
  "3gp",
]);
const AUDIO_EXTS = new Set([
  "flac",
  "mp3",
  "aac",
  "ogg",
  "wav",
  "wma",
  "ape",
  "alac",
  "m4a",
  "opus",
  "aiff",
  "dsd",
  "dsf",
  "dff",
  "ac3",
  "dts",
  "thd",
]);
const IMAGE_EXTS = new Set([
  "jpg",
  "jpeg",
  "png",
  "gif",
  "bmp",
  "webp",
  "svg",
  "tiff",
  "tif",
  "ico",
  "heic",
  "heif",
  "avif",
  "jxl",
  "raw",
  "cr2",
  "nef",
  "arw",
  "dng",
  "psd",
  "ai",
  "eps",
]);
const SUB_EXTS = new Set([
  "srt",
  "ass",
  "ssa",
  "sub",
  "idx",
  "sup",
  "vtt",
  "smi",
  "lrc",
]);
const EBOOK_EXTS = new Set([
  "epub",
  "mobi",
  "azw",
  "azw3",
  "pdf",
  "djvu",
  "cbz",
  "cbr",
  "fb2",
]);
const DOCUMENT_EXTS = new Set([
  "txt",
  "doc",
  "docx",
  "xls",
  "xlsx",
  "ppt",
  "pptx",
  "md",
  "nfo",
  "csv",
  "json",
  "xml",
  "html",
  "htm",
  "rtf",
]);
const ARCHIVE_EXTS = new Set([
  "zip",
  "rar",
  "7z",
  "tar",
  "gz",
  "bz2",
  "xz",
  "zst",
  "tgz",
]);

function classifyFileType(path: string): FileCategory {
  const ext = (path.split(".").pop() ?? "").toLowerCase();
  if (VIDEO_EXTS.has(ext)) return "video";
  if (AUDIO_EXTS.has(ext)) return "audio";
  if (IMAGE_EXTS.has(ext)) return "image";
  if (SUB_EXTS.has(ext)) return "subtitle";
  if (EBOOK_EXTS.has(ext)) return "ebook";
  if (DOCUMENT_EXTS.has(ext)) return "document";
  if (ARCHIVE_EXTS.has(ext)) return "archive";
  return "other";
}

function getExtFromPath(path: string): string {
  const ext = (path.split(".").pop() ?? "").toLowerCase();
  return ext ? `.${ext}` : "";
}

const SE_EP_RE = /S(\d{1,4})(?:\s*E(\d{1,4})(?:\s*-\s*E?(\d{1,4}))?)?/gi;

function parseSeasonEpisode(path: string): {
  season: number | null;
  episodes: number[];
} {
  let season: number | null = null;
  const episodes: number[] = [];
  let m: RegExpExecArray | null;
  // biome-ignore lint/suspicious/noAssignInExpressions: regex exec loop
  while ((m = SE_EP_RE.exec(path)) !== null) {
    if (m[1]) season = Number.parseInt(m[1], 10);
    if (m[2]) {
      const start = Number.parseInt(m[2], 10);
      const end = m[3] ? Number.parseInt(m[3], 10) : start;
      for (let e = start; e <= end; e++) {
        if (!episodes.includes(e)) episodes.push(e);
      }
    }
  }
  return { season, episodes };
}

export default function DownloadConfirmWindow({ win }: Props) {
  const torrent = win.metadata.torrent as PtSearchResultWithSite;
  const onSuccess = win.metadata.onSuccess as (() => void) | undefined;

  const [form] = Form.useForm();
  const [selectedFiles, setSelectedFiles] = useState<Map<number, boolean>>(
    new Map(),
  );
  const queryClient = useQueryClient();

  // 下载器列表
  const clientsQuery = useQuery({
    queryKey: ["download-clients"],
    queryFn: () => downloadsApi.clients.list(),
  });
  const clients = clientsQuery.data ?? [];
  const defaultClient = clients.find((c) => c.isDefault) ?? clients[0];

  // 解析保存路径
  const resolvePathQuery = useQuery({
    queryKey: ["resolve-path", defaultClient?.id, torrent.category],
    queryFn: () =>
      downloadsApi.torrent.resolvePath({
        clientId: defaultClient!.id,
        category: torrent.category || undefined,
      }),
    enabled: !!defaultClient,
  });
  const resolvedPath = resolvePathQuery.data?.path ?? null;

  // 预览种子文件
  const previewMutation = useMutation({
    mutationFn: () =>
      downloadsApi.torrent.preview({
        siteId: torrent.siteDbId,
        torrentId: torrent.id,
      }),
  });

  // 提交下载
  const downloadMutation = useMutation({
    mutationFn: (values: {
      clientId: string;
      savePath?: string;
      season?: number;
      episodes?: number[];
    }) =>
      downloadsApi.torrent.downloadFiltered({
        clientId: values.clientId,
        siteId: torrent.siteDbId,
        torrentId: torrent.id,
        savePath: values.savePath,
        season: values.season,
        episodes: values.episodes,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["torrents"] });
      onSuccess?.();
      win.close();
    },
  });

  const files = previewMutation.data ?? [];

  // 文件类型标注
  const enrichedFiles = useMemo(
    () =>
      files.map((f) => ({
        ...f,
        fileType: classifyFileType(f.path),
        ext: getExtFromPath(f.path),
      })),
    [files],
  );

  // 初始化选择状态
  useEffect(() => {
    if (previewMutation.data) {
      const map = new Map<number, boolean>();
      for (const f of previewMutation.data) map.set(f.index, f.selected);
      setSelectedFiles(map);
    }
  }, [previewMutation.data]);

  // 设置默认下载器
  useEffect(() => {
    if (defaultClient && !form.getFieldValue("clientId")) {
      form.setFieldValue("clientId", defaultClient.id);
    }
  }, [defaultClient, form]);

  // 自动触发预览
  useEffect(() => {
    previewMutation.mutate();
    // biome-ignore lint/correctness/useExhaustiveDependencies: mount only
  }, [previewMutation.mutate]);

  // 自动集数过滤
  const autoFilter = useMemo(
    () => parseSeasonEpisode(torrent.title),
    [torrent.title],
  );

  // 文件统计
  const fileStats = useMemo(() => {
    const total = files.length;
    const selected = files.filter(
      (f) => selectedFiles.get(f.index) ?? f.selected,
    ).length;
    const selectedSize = files
      .filter((f) => selectedFiles.get(f.index) ?? f.selected)
      .reduce((s, f) => s + f.size, 0);
    const totalSize = files.reduce((s, f) => s + f.size, 0);
    return { total, selected, selectedSize, totalSize };
  }, [files, selectedFiles]);

  const handleToggle = (index: number, checked: boolean) => {
    setSelectedFiles((prev) => {
      const next = new Map(prev);
      next.set(index, checked);
      return next;
    });
  };
  const handleSelectAll = () => {
    setSelectedFiles((prev) => {
      const next = new Map(prev);
      for (const f of files) next.set(f.index, true);
      return next;
    });
  };
  const handleDeselectAll = () => {
    setSelectedFiles((prev) => {
      const next = new Map(prev);
      for (const f of files) next.set(f.index, false);
      return next;
    });
  };
  const applyAutoFilter = () => {
    if (!autoFilter.season || !previewMutation.data) return;
    const newMap = new Map<number, boolean>();
    for (const f of previewMutation.data) {
      const se = parseSeasonEpisode(f.path);
      let include = true;
      if (se.season && se.season !== autoFilter.season) include = false;
      if (
        autoFilter.episodes.length > 0 &&
        se.episodes.length > 0 &&
        !se.episodes.some((e) => autoFilter.episodes.includes(e))
      )
        include = false;
      newMap.set(f.index, include);
    }
    setSelectedFiles(newMap);
  };

  const handleSubmit = async () => {
    const values = await form.validateFields();
    downloadMutation.mutate({
      clientId: values.clientId,
      savePath: values.savePath || undefined,
      season: autoFilter.season ?? undefined,
      episodes:
        autoFilter.episodes.length > 0 ? autoFilter.episodes : undefined,
    });
  };

  const isMultiFile = files.length > 1;

  return (
    <div className="flex flex-col h-full">
      {/* 种子信息卡片 */}
      <div className="px-4 py-3 border-b border-base shrink-0">
        <div className="rounded-xl border border-border-base bg-surface-base/40 p-3.5">
          <TorrentCardBody torrent={torrent} />
        </div>
      </div>

      {/* 表单 + 文件列表 */}
      <div className="flex-1 min-h-0 overflow-y-auto">
        <div className="px-4 py-3 space-y-4">
          <Form form={form} layout="vertical" size="small" autoComplete="off">
            <div className="grid grid-cols-2 gap-x-4">
              <Form.Item
                name="clientId"
                label="下载器"
                rules={[{ required: true, message: "请选择下载器" }]}
              >
                <Select
                  loading={clientsQuery.isLoading}
                  placeholder="选择下载器"
                >
                  {clients.map((c) => (
                    <Select.Option key={c.id} value={c.id}>
                      {c.name} {c.isDefault && "(默认)"}
                    </Select.Option>
                  ))}
                </Select>
              </Form.Item>
              <Form.Item label="保存路径">
                <div className="min-h-8 flex items-center px-3 rounded-md border border-black/[0.08] dark:border-white/[0.1] bg-[var(--color-surface-sunken)] text-sm text-fg-muted">
                  {resolvePathQuery.isLoading
                    ? "解析中..."
                    : resolvedPath || "使用下载器默认路径"}
                </div>
              </Form.Item>
            </div>
          </Form>

          {/* 集数过滤提示 */}
          {autoFilter.season && (
            <div className="flex items-center gap-2 p-2 rounded bg-accent/10 text-sm">
              <span className="text-fg-muted">自动过滤:</span>
              <Tag>S{String(autoFilter.season).padStart(2, "0")}</Tag>
              {autoFilter.episodes.length > 0 && (
                <span className="text-fg-muted">
                  E
                  {autoFilter.episodes
                    .map((e) => String(e).padStart(2, "0"))
                    .join(",E")}
                </span>
              )}
              <Button size="small" onClick={applyAutoFilter}>
                应用
              </Button>
            </div>
          )}

          {/* 文件选择面板 */}
          <Collapse
            size="small"
            defaultActiveKey={["files"]}
            items={[
              {
                key: "files",
                label: (
                  <div className="flex items-center gap-2">
                    <span>{isMultiFile ? "文件选择" : "文件预览"}</span>
                    {fileStats.total > 0 && (
                      <Tag>
                        {fileStats.selected}/{fileStats.total}
                      </Tag>
                    )}
                  </div>
                ),
                children: previewMutation.isPending ? (
                  <div className="flex items-center justify-center py-4">
                    <Spin size="small" />
                    <span className="text-fg-muted ml-2">解析种子文件...</span>
                  </div>
                ) : previewMutation.isError ? (
                  <div className="text-red-500 text-sm py-4">
                    解析失败: {(previewMutation.error as Error)?.message}
                  </div>
                ) : files.length === 0 ? null : (
                  <FileList
                    files={enrichedFiles}
                    isMultiFile={isMultiFile}
                    selectedFiles={selectedFiles}
                    fileStats={fileStats}
                    onToggle={handleToggle}
                    onSelectAll={handleSelectAll}
                    onDeselectAll={handleDeselectAll}
                  />
                ),
              },
            ]}
          />
        </div>
      </div>

      {/* 底部按钮 */}
      <div className="flex items-center justify-between px-4 py-3 border-t border-base shrink-0">
        <Button onClick={win.close}>取消</Button>
        <Button
          variant="primary"
          onClick={handleSubmit}
          loading={downloadMutation.isPending || previewMutation.isPending}
          disabled={
            previewMutation.isPending ||
            downloadMutation.isPending ||
            (files.length > 0 && fileStats.selected === 0)
          }
        >
          下载 ({fileStats.selected} 文件)
        </Button>
      </div>
    </div>
  );
}

// ── 文件列表（带排序） ──────────────────────────────────────────────────────

type SortKey = "name" | "ext" | "type" | "size";
type SortDir = "asc" | "desc";

function SortColHeader({
  label,
  active,
  dir,
  onClick,
  className,
}: {
  label: string;
  active: boolean;
  dir: SortDir;
  onClick: () => void;
  className?: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`inline-flex items-center gap-0.5 transition-colors ${
        active
          ? "text-[var(--color-accent)]"
          : "text-[var(--color-fg-muted)] hover:text-[var(--color-fg-secondary)]"
      } ${className ?? ""}`}
    >
      {label}
      {active ? (
        dir === "asc" ? (
          <ArrowUp className="h-3 w-3" />
        ) : (
          <ArrowDown className="h-3 w-3" />
        )
      ) : (
        <ChevronsUpDown className="h-3 w-3 opacity-40" />
      )}
    </button>
  );
}

function FileList({
  files,
  isMultiFile,
  selectedFiles,
  fileStats,
  onToggle,
  onSelectAll,
  onDeselectAll,
}: {
  files: (PreviewFileItem & { fileType: FileCategory; ext: string })[];
  isMultiFile: boolean;
  selectedFiles: Map<number, boolean>;
  fileStats: { selectedSize: number; totalSize: number };
  onToggle: (index: number, checked: boolean) => void;
  onSelectAll: () => void;
  onDeselectAll: () => void;
}) {
  const parentRef = useRef<HTMLDivElement>(null);
  const [sortKey, setSortKey] = useState<SortKey>("name");
  const [sortDir, setSortDir] = useState<SortDir>("asc");
  const _ROW_HEIGHT = 33;

  const allChecked =
    files.length > 0 &&
    files.every((f) => selectedFiles.get(f.index) ?? f.selected);
  const someChecked =
    !allChecked && files.some((f) => selectedFiles.get(f.index) ?? f.selected);

  const handleSort = (key: SortKey) => {
    if (sortKey === key) setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    else {
      setSortKey(key);
      setSortDir("asc");
    }
  };

  const sortedFiles = useMemo(() => {
    return [...files].sort((a, b) => {
      let cmp = 0;
      if (sortKey === "name") cmp = a.path.localeCompare(b.path);
      else if (sortKey === "ext") cmp = a.ext.localeCompare(b.ext);
      else if (sortKey === "type")
        cmp = (a.fileType ?? "").localeCompare(b.fileType ?? "");
      else if (sortKey === "size") cmp = a.size - b.size;
      return sortDir === "asc" ? cmp : -cmp;
    });
  }, [files, sortKey, sortDir]);

  return (
    <div>
      {/* 表头 */}
      <div className="flex items-center gap-2 border-b-2 border-border-base pb-1.5 text-xs select-none">
        {isMultiFile && (
          <Checkbox
            checked={allChecked}
            indeterminate={someChecked}
            onChange={() => {
              if (allChecked) onDeselectAll();
              else onSelectAll();
            }}
          />
        )}
        <div className="flex-1 min-w-0">
          <SortColHeader
            label="文件名"
            active={sortKey === "name"}
            dir={sortDir}
            onClick={() => handleSort("name")}
          />
        </div>
        <div className="shrink-0 flex items-center justify-end gap-2">
          <div className="w-10 text-right">
            <SortColHeader
              label="扩展名"
              active={sortKey === "ext"}
              dir={sortDir}
              onClick={() => handleSort("ext")}
            />
          </div>
          <div className="w-12 text-right">
            <SortColHeader
              label="类型"
              active={sortKey === "type"}
              dir={sortDir}
              onClick={() => handleSort("type")}
            />
          </div>
          <div className="w-16 text-right">
            <SortColHeader
              label="大小"
              active={sortKey === "size"}
              dir={sortDir}
              onClick={() => handleSort("size")}
            />
          </div>
        </div>
      </div>
      {/* 文件行 */}
      <div ref={parentRef} className="overflow-auto max-h-[280px]">
        {sortedFiles.map((file) => {
          const checked = selectedFiles.get(file.index) ?? file.selected;
          const filename = file.path.split("/").pop() ?? file.path;
          const dir = file.path.includes("/")
            ? file.path.substring(0, file.path.lastIndexOf("/"))
            : null;

          return (
            <div
              key={file.index}
              className={`flex items-center gap-2 border-b border-border-base py-1.5 last:border-b-0 ${!checked ? "opacity-50" : ""}`}
            >
              {isMultiFile && (
                <Checkbox
                  checked={checked}
                  onChange={(e) => onToggle(file.index, e.target.checked)}
                />
              )}
              <div className="min-w-0 flex-1">
                <Tooltip title={file.path}>
                  <span
                    className={`block truncate text-xs ${!checked ? "text-fg-muted line-through" : ""}`}
                  >
                    {dir && (
                      <span className="text-fg-muted text-xs">{dir}/</span>
                    )}
                    {filename}
                  </span>
                </Tooltip>
              </div>
              <div className="shrink-0 flex items-center justify-end gap-2">
                <span className="text-fg-muted text-xs w-10 text-right tabular-nums">
                  {file.ext}
                </span>
                <Tag
                  color={fileTypeColor[file.fileType] ?? "default"}
                  className="text-xs !m-0 w-12 justify-center"
                >
                  {fileTypeLabel[file.fileType]}
                </Tag>
                <span className="text-fg-muted text-xs w-16 text-right tabular-nums">
                  {formatSize(file.size)}
                </span>
              </div>
            </div>
          );
        })}
      </div>
      {/* 底部统计 */}
      {isMultiFile && (
        <div className="pt-1 text-right text-xs text-fg-muted tabular-nums">
          {formatSize(fileStats.selectedSize)} /{" "}
          {formatSize(fileStats.totalSize)}
        </div>
      )}
    </div>
  );
}
