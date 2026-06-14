import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Button,
  Card,
  CloudDownloadOutlined,
  CloudUploadOutlined,
  DeleteOutlined,
  HddOutlined,
  Input,
  Modal,
  PauseCircleOutlined,
  PlayCircleOutlined,
  Popconfirm,
  Progress,
  ReloadOutlined,
  SearchOutlined,
  Select,
  Statistic,
  Table,
  Tag,
  Tooltip,
  useContextMenu,
  useDateFormat,
} from "@tokimo/ui";
import { Pause, Play, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { downloadsApi, type TorrentInfoDto } from "../api/client";

interface TorrentListWindowProps {
  clientId: string;
  clientName: string;
  clientType: string;
}

const stateColors: Record<string, string> = {
  downloading: "processing",
  seeding: "success",
  pausedDL: "warning",
  pausedUP: "warning",
  stalledDL: "default",
  stalledUP: "default",
  checkingDL: "processing",
  checkingUP: "processing",
  queuedDL: "default",
  queuedUP: "default",
  error: "error",
  missingFiles: "error",
  uploading: "processing",
  completed: "success",
  unknown: "default",
};

const formatSize = (bytes: number): string => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / k ** i).toFixed(2)} ${sizes[i]}`;
};

const formatSpeed = (bytesPerSecond: number): string => {
  return `${formatSize(bytesPerSecond)}/s`;
};

const formatEta = (seconds?: number | null): string => {
  if (!seconds || seconds < 0 || seconds > 8640000) return "∞";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
};

export function TorrentListWindow({ clientId }: TorrentListWindowProps) {
  const { t } = useTranslation();
  const { formatLong } = useDateFormat();
  const [selectedRowKeys, setSelectedRowKeys] = useState<string[]>([]);
  const [searchText, setSearchText] = useState("");
  const [stateFilter, setStateFilter] = useState<string>("all");
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);

  const { open: openContextMenu, contextMenu } = useContextMenu();

  const utils = useQueryClient();

  const torrentsQuery = useQuery({
    queryKey: ["downloads", "torrents", clientId],
    queryFn: () => downloadsApi.torrents.list(clientId),
    refetchInterval: 3000,
  });

  const transferQuery = useQuery({
    queryKey: ["downloads", "transfer", clientId],
    queryFn: () => downloadsApi.torrents.transferInfo(clientId),
    refetchInterval: 3000,
  });

  const pauseMutation = useMutation({
    mutationFn: (hashes: string[]) =>
      downloadsApi.torrents.pause(clientId, hashes),
    onSuccess: () => {
      void utils.invalidateQueries({
        queryKey: ["downloads", "torrents", clientId],
      });
      setSelectedRowKeys([]);
    },
  });

  const resumeMutation = useMutation({
    mutationFn: (hashes: string[]) =>
      downloadsApi.torrents.resume(clientId, hashes),
    onSuccess: () => {
      void utils.invalidateQueries({
        queryKey: ["downloads", "torrents", clientId],
      });
      setSelectedRowKeys([]);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: ({
      hashes,
      deleteFiles,
    }: {
      hashes: string[];
      deleteFiles: boolean;
    }) => downloadsApi.torrents.delete(clientId, hashes, deleteFiles),
    onSuccess: () => {
      void utils.invalidateQueries({
        queryKey: ["downloads", "torrents", clientId],
      });
      setSelectedRowKeys([]);
    },
  });

  const filteredData = useMemo(() => {
    let data = torrentsQuery.data ?? [];

    if (searchText) {
      const lowerSearch = searchText.toLowerCase();
      data = data.filter(
        (item) =>
          item.name.toLowerCase().includes(lowerSearch) ||
          item.category?.toLowerCase().includes(lowerSearch) ||
          item.savePath.toLowerCase().includes(lowerSearch),
      );
    }

    if (stateFilter !== "all") {
      data = data.filter((item) => item.state === stateFilter);
    }

    return data;
  }, [torrentsQuery.data, searchText, stateFilter]);

  const stats = useMemo(() => {
    const torrents = torrentsQuery.data ?? [];
    return {
      total: torrents.length,
      downloading: torrents.filter((item) => item.state === "downloading")
        .length,
      seeding: torrents.filter((item) =>
        ["seeding", "uploading"].includes(item.state),
      ).length,
      paused: torrents.filter((item) =>
        ["pausedDL", "pausedUP"].includes(item.state),
      ).length,
    };
  }, [torrentsQuery.data]);

  const handlePause = (hashes: string[]) => {
    if (hashes.length === 0) return;
    pauseMutation.mutate(hashes);
  };

  const handleResume = (hashes: string[]) => {
    if (hashes.length === 0) return;
    resumeMutation.mutate(hashes);
  };

  const handleDelete = (hashes: string[], deleteFiles = false) => {
    if (hashes.length === 0) return;
    deleteMutation.mutate({ hashes, deleteFiles });
  };

  const buildContextMenuItems = (record: TorrentInfoDto) => {
    const isPaused = ["pausedDL", "pausedUP"].includes(record.state);
    return [
      isPaused
        ? {
            key: "resume",
            label: t("media.torrents.resume"),
            icon: <Play size={13} />,
            onClick: () => handleResume([record.hash]),
          }
        : {
            key: "pause",
            label: t("media.torrents.pause"),
            icon: <Pause size={13} />,
            onClick: () => handlePause([record.hash]),
          },
      { key: "divider", type: "divider" as const },
      {
        key: "delete",
        label: t("media.torrents.deleteTaskOnly"),
        icon: <Trash2 size={13} />,
        danger: true,
        onClick: () => handleDelete([record.hash], false),
      },
      {
        key: "deleteWithFiles",
        label: t("media.torrents.deleteWithFiles"),
        icon: <Trash2 size={13} />,
        danger: true,
        onClick: () => setDeleteTarget(record.hash),
      },
    ];
  };

  const allStates = [
    "downloading",
    "seeding",
    "pausedDL",
    "pausedUP",
    "stalledDL",
    "stalledUP",
    "queuedDL",
    "queuedUP",
    "checkingDL",
    "uploading",
    "completed",
    "error",
    "missingFiles",
  ];

  const columns = [
    {
      title: t("media.torrents.name"),
      dataIndex: "name",
      key: "name",
      ellipsis: true,
      width: 280,
      minWidth: 180,
      render: (name: string, record: TorrentInfoDto) => (
        <Tooltip title={`${t("media.torrents.savePath")}: ${record.savePath}`}>
          <span className="block truncate" style={{ maxWidth: "100%" }}>
            {name}
          </span>
        </Tooltip>
      ),
    },
    {
      title: t("media.torrents.size"),
      dataIndex: "size",
      key: "size",
      width: 100,
      sorter: (a: TorrentInfoDto, b: TorrentInfoDto) => a.size - b.size,
      render: (size: number) => formatSize(size),
    },
    {
      title: t("media.torrents.progress"),
      dataIndex: "progress",
      key: "progress",
      width: 120,
      sorter: (a: TorrentInfoDto, b: TorrentInfoDto) => a.progress - b.progress,
      render: (progress: number) => (
        <Progress
          percent={Math.round(progress * 100)}
          size="small"
          status={progress >= 1 ? "success" : "active"}
          format={(p) => `${p}%`}
        />
      ),
    },
    {
      title: t("media.torrents.state"),
      dataIndex: "state",
      key: "state",
      width: 100,
      filters: allStates.map((s) => ({
        text: t(`media.torrents.state.${s}`, s),
        value: s,
      })),
      onFilter: (value: boolean | React.Key, record: TorrentInfoDto) =>
        record.state === value,
      render: (state: string) => (
        <Tag color={stateColors[state] ?? "default"}>
          {t(`media.torrents.state.${state}`, state)}
        </Tag>
      ),
    },
    {
      title: t("media.torrents.downloadSpeed"),
      dataIndex: "downloadSpeed",
      key: "downloadSpeed",
      width: 110,
      sorter: (a: TorrentInfoDto, b: TorrentInfoDto) =>
        a.downloadSpeed - b.downloadSpeed,
      render: (speed: number) => (
        <span className={speed > 0 ? "text-green-500" : "text-fg-muted"}>
          {formatSpeed(speed)}
        </span>
      ),
    },
    {
      title: t("media.torrents.uploadSpeed"),
      dataIndex: "uploadSpeed",
      key: "uploadSpeed",
      width: 110,
      sorter: (a: TorrentInfoDto, b: TorrentInfoDto) =>
        a.uploadSpeed - b.uploadSpeed,
      render: (speed: number) => (
        <span className={speed > 0 ? "text-yellow-500" : "text-fg-muted"}>
          {formatSpeed(speed)}
        </span>
      ),
    },
    {
      title: t("media.torrents.ratio"),
      dataIndex: "ratio",
      key: "ratio",
      width: 80,
      sorter: (a: TorrentInfoDto, b: TorrentInfoDto) => a.ratio - b.ratio,
      render: (ratio: number) => (
        <span className={ratio >= 1 ? "text-green-500" : "text-fg-muted"}>
          {ratio.toFixed(2)}
        </span>
      ),
    },
    {
      title: t("media.torrents.eta"),
      dataIndex: "eta",
      key: "eta",
      width: 95,
      render: (eta?: number | null) => formatEta(eta),
    },
    {
      title: t("media.torrents.addedTime"),
      dataIndex: "addedOn",
      key: "addedOn",
      width: 150,
      sorter: (a: TorrentInfoDto, b: TorrentInfoDto) => a.addedOn - b.addedOn,
      render: (timestamp: number) => formatLong(timestamp * 1000),
    },
  ];

  const rowSelection = {
    selectedRowKeys,
    onChange: (keys: React.Key[]) => setSelectedRowKeys(keys as string[]),
  };

  return (
    <div className="flex flex-col h-full gap-4 p-4">
      <div className="grid grid-cols-4 gap-4 shrink-0">
        <Card size="small">
          <Statistic
            title={t("media.torrents.total")}
            value={stats.total}
            suffix={`(${filteredData.length})`}
          />
        </Card>
        <Card size="small">
          <Statistic
            title={t("media.torrents.globalDlSpeed")}
            value={formatSpeed(transferQuery.data?.dlSpeed ?? 0)}
            prefix={<CloudDownloadOutlined style={{ color: "#52c41a" }} />}
          />
        </Card>
        <Card size="small">
          <Statistic
            title={t("media.torrents.globalUpSpeed")}
            value={formatSpeed(transferQuery.data?.upSpeed ?? 0)}
            prefix={<CloudUploadOutlined style={{ color: "#faad14" }} />}
          />
        </Card>
        <Card size="small">
          <Statistic
            title={t("media.torrents.freeSpace")}
            value={formatSize(transferQuery.data?.freeSpace ?? 0)}
            prefix={<HddOutlined style={{ color: "#1890ff" }} />}
          />
        </Card>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-4 shrink-0">
        <div className="flex items-center gap-2 flex-wrap">
          <Input
            placeholder={t("media.torrents.search")}
            prefix={<SearchOutlined />}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            style={{ width: 250 }}
            allowClear
          />
          <Select
            value={stateFilter}
            onChange={setStateFilter}
            style={{ width: 140 }}
            options={[
              { label: t("media.torrents.allStates"), value: "all" },
              {
                label: t("media.torrents.downloading"),
                value: "downloading",
              },
              {
                label: t("media.torrents.seeding"),
                value: "seeding",
              },
              { label: t("media.torrents.paused"), value: "pausedDL" },
              {
                label: t("media.torrents.completed"),
                value: "completed",
              },
              { label: t("media.torrents.error"), value: "error" },
            ]}
          />
          <Button
            icon={<ReloadOutlined />}
            onClick={() => torrentsQuery.refetch()}
            loading={torrentsQuery.isRefetching}
          >
            {t("media.common.refresh")}
          </Button>
        </div>

        {selectedRowKeys.length > 0 && (
          <div className="flex items-center gap-2">
            <span className="text-fg-muted">
              {t("media.torrents.selected", {
                count: selectedRowKeys.length,
              })}
            </span>
            <Button
              icon={<PlayCircleOutlined />}
              onClick={() => handleResume(selectedRowKeys)}
              loading={resumeMutation.isPending}
            >
              {t("media.torrents.resume")}
            </Button>
            <Button
              icon={<PauseCircleOutlined />}
              onClick={() => handlePause(selectedRowKeys)}
              loading={pauseMutation.isPending}
            >
              {t("media.torrents.pause")}
            </Button>
            <Popconfirm
              title={t("media.torrents.delete")}
              description={`${selectedRowKeys.length} items`}
              onConfirm={() => handleDelete(selectedRowKeys, false)}
              okText={t("media.common.confirm")}
              cancelText={t("media.common.cancel")}
            >
              <Button variant="danger" icon={<DeleteOutlined />}>
                {t("media.torrents.delete")}
              </Button>
            </Popconfirm>
          </div>
        )}
      </div>

      <div className="flex-1 min-h-0">
        <Table
          dataSource={filteredData}
          columns={columns}
          rowKey="hash"
          rowSelection={rowSelection}
          loading={torrentsQuery.isLoading}
          virtual
          itemHeight={33}
          scroll={{ x: 1180, y: "100%" }}
          style={{ height: "100%" }}
          size="small"
          onRow={(record) => ({
            onContextMenu: (e) =>
              openContextMenu(
                e,
                buildContextMenuItems(record as TorrentInfoDto),
              ),
            className: "cursor-context-menu",
          })}
        />
      </div>

      <Modal
        open={deleteTarget !== null}
        title={t("media.torrents.delete")}
        okText={t("media.torrents.deleteWithFiles")}
        cancelText={t("media.common.cancel")}
        okButtonProps={{ danger: true }}
        onOk={(): undefined => {
          if (deleteTarget) handleDelete([deleteTarget], true);
          setDeleteTarget(null);
        }}
        onCancel={() => setDeleteTarget(null)}
      >
        <p className="text-sm text-[var(--color-fg-secondary)]">
          {t("media.torrents.deleteWithFiles")}
        </p>
      </Modal>

      {contextMenu}
    </div>
  );
}
