import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useWindowActions } from "@tokimo/sdk";
import {
  Button,
  DeleteOutlined,
  EditOutlined,
  EyeOutlined,
  Modal,
  ReloadOutlined,
  Table,
  Tag,
} from "@tokimo/ui";
import { Plus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type DownloadClientDto,
  type DownloadClientType,
  downloadsApi,
} from "../api/client";

// ── Type labels & colors ──────────────────────────────────────────────────────

const clientTypeLabels: Record<DownloadClientType, string> = {
  qbittorrent: "qBittorrent",
  transmission: "Transmission",
  aria2: "Aria2",
  deluge: "Deluge",
  rtorrent: "rTorrent",
  synology: "Synology Download Station",
  xunlei: "迅雷远程下载",
  pan115: "115网盘",
};

const clientTypeColors: Record<DownloadClientType, string> = {
  qbittorrent: "blue",
  transmission: "red",
  aria2: "cyan",
  deluge: "green",
  rtorrent: "purple",
  synology: "geekblue",
  xunlei: "orange",
  pan115: "gold",
};

// ── Connection status cell ────────────────────────────────────────────────────

function ConnectionStatusCell({
  status,
  isLoading,
}: {
  status: { isConnected: boolean; errorMessage?: string } | undefined;
  isLoading: boolean;
}) {
  const { t } = useTranslation();

  if (isLoading) {
    return (
      <span className="flex items-center gap-1.5">
        <span className="inline-block h-2 w-2 animate-pulse rounded-full bg-gray-400" />
        <span className="text-fg-muted text-sm">
          {t("media.downloadClients.testing")}
        </span>
      </span>
    );
  }

  if (!status) {
    return (
      <span className="flex items-center gap-1.5">
        <span className="inline-block h-2 w-2 rounded-full bg-red-500" />
        <span className="text-sm text-red-600 dark:text-red-400">
          {t("media.downloadClients.disconnected")}
        </span>
      </span>
    );
  }

  return (
    <span className="flex items-center gap-1.5">
      <span
        className={`inline-block h-2 w-2 rounded-full ${
          status.isConnected ? "bg-green-500" : "bg-red-500"
        }`}
      />
      <span
        className={`text-sm ${
          status.isConnected
            ? "text-green-600 dark:text-green-400"
            : "text-red-600 dark:text-red-400"
        }`}
      >
        {status.isConnected
          ? t("media.downloadClients.connected")
          : status.errorMessage || t("media.downloadClients.disconnected")}
      </span>
    </span>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

export default function DownloadClientsPage() {
  const { t } = useTranslation();
  const utils = useQueryClient();
  const windowActions = useWindowActions();
  const dc = "media.downloadClients";

  // ── Queries ──
  const clientsQuery = useQuery({
    queryKey: ["downloads", "clients"],
    queryFn: () => downloadsApi.clients.list(),
  });

  const allStatusQuery = useQuery({
    queryKey: ["downloads", "clients", "status"],
    queryFn: () => downloadsApi.clients.allStatus(),
  });

  // ── Mutations ──
  const deleteMutation = useMutation({
    mutationFn: (id: string) => downloadsApi.clients.delete(id),
    onSuccess: () => {
      void utils.invalidateQueries({ queryKey: ["downloads", "clients"] });
    },
  });

  const setDefaultMutation = useMutation({
    mutationFn: (id: string) => downloadsApi.clients.setDefault(id),
    onSuccess: () => {
      void utils.invalidateQueries({ queryKey: ["downloads", "clients"] });
    },
  });

  const reorderMutation = useMutation({
    mutationFn: (items: { id: string; sortOrder: number }[]) =>
      downloadsApi.clients.reorder(items),
    onSuccess: () => {
      void utils.invalidateQueries({ queryKey: ["downloads", "clients"] });
      setOptimisticClients(null);
    },
    onError: () => {
      setOptimisticClients(null);
    },
  });

  const [optimisticClients, setOptimisticClients] = useState<
    DownloadClientDto[] | null
  >(null);

  const handleReorder = (reordered: DownloadClientDto[]) => {
    setOptimisticClients(reordered);
    reorderMutation.mutate(
      reordered.map((c, i) => ({ id: c.id, sortOrder: i })),
    );
  };

  // ── Handlers ──
  const handleViewTorrents = (client: DownloadClientDto) => {
    windowActions.openModalWindow({
      component: () => import("./TorrentListModalWindow"),
      title: `${client.name} — ${t("media.torrents.title")}`,
      width: 1000,
      height: 650,
      metadata: {
        clientId: client.id,
        clientName: client.name,
        clientType: client.type,
      },
    });
  };

  const handleOpenModal = (editClient?: DownloadClientDto) => {
    windowActions.openModalWindow({
      component: () => import("./ClientFormWindow"),
      title: editClient ? t(`${dc}.edit`) : t(`${dc}.add`),
      width: 760,
      height: 600,
      metadata: { editingClient: editClient ?? null },
    });
  };

  const handleDelete = (client: DownloadClientDto) => {
    Modal.confirm({
      title: t(`${dc}.deleteConfirmTitle`),
      content: t(`${dc}.deleteConfirmContent`, { name: client.name }),
      okText: t("media.common.delete"),
      variant: "danger",
      cancelText: t("media.common.cancel"),
      onOk: () => deleteMutation.mutateAsync(client.id),
    });
  };

  // ── Status map ──
  const statusMap = new Map((allStatusQuery.data ?? []).map((s) => [s.id, s]));

  // ── Columns ──
  const columns = [
    {
      title: t(`${dc}.name`),
      dataIndex: "name",
      key: "name",
      render: (name: string) => <span className="font-semibold">{name}</span>,
    },
    {
      title: t(`${dc}.type`),
      dataIndex: "type",
      key: "type",
      render: (type: DownloadClientType) => (
        <Tag color={clientTypeColors[type]}>{clientTypeLabels[type]}</Tag>
      ),
    },
    {
      title: t(`${dc}.accessUrl`),
      dataIndex: "url",
      key: "url",
      render: (url: string) => (
        <a
          className="text-[var(--color-accent-text)] hover:text-[var(--color-accent)]"
          href={url}
          target="_blank"
          rel="noreferrer"
        >
          {url}
        </a>
      ),
    },
    {
      title: t(`${dc}.connectionStatus`),
      key: "connectionStatus",
      render: (_: unknown, record: DownloadClientDto) => (
        <ConnectionStatusCell
          status={statusMap.get(record.id)}
          isLoading={allStatusQuery.isLoading}
        />
      ),
    },
    {
      title: t(`${dc}.default`),
      key: "isDefault",
      render: (_: unknown, record: DownloadClientDto) =>
        record.isDefault ? (
          <span className="inline-flex items-center rounded-full border border-yellow-200 bg-yellow-50 px-2.5 py-0.5 text-xs font-medium text-yellow-700 dark:border-yellow-700 dark:bg-yellow-900/20 dark:text-yellow-400">
            {t(`${dc}.default`)}
          </span>
        ) : (
          <Button
            variant="unstyled"
            size="small"
            shape="round"
            onClick={() => setDefaultMutation.mutate(record.id)}
            disabled={setDefaultMutation.isPending}
            className="border-border-base bg-fill-tertiary text-fg-muted hover:border-yellow-200 hover:bg-yellow-50 hover:text-yellow-700 dark:hover:border-yellow-700 dark:hover:bg-yellow-900/20 dark:hover:text-yellow-400"
          >
            {t(`${dc}.setDefault`)}
          </Button>
        ),
    },
    {
      title: t("media.common.actions"),
      key: "actions",
      width: 120,
      render: (_: unknown, record: DownloadClientDto) => (
        <div className="flex items-center gap-1">
          <Button
            variant="text"
            icon={<EyeOutlined />}
            onClick={() => handleViewTorrents(record)}
            title={t(`${dc}.viewTorrents`)}
          />
          <Button
            variant="text"
            icon={<EditOutlined />}
            onClick={() => handleOpenModal(record)}
          />
          <Button
            variant="text"
            danger
            icon={<DeleteOutlined />}
            onClick={() => handleDelete(record)}
          />
        </div>
      ),
    },
  ];

  // ── Render ──
  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-base">
        <div>
          <h1 className="text-lg font-semibold">{t(`${dc}.intro.headline`)}</h1>
          <p className="text-sm text-fg-muted mt-0.5">
            {t(`${dc}.intro.description`)}
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            icon={<ReloadOutlined />}
            onClick={() => {
              void clientsQuery.refetch();
              void allStatusQuery.refetch();
            }}
            loading={clientsQuery.isRefetching}
          >
            {t(`${dc}.refresh`)}
          </Button>
          <Button
            variant="primary"
            icon={<Plus size={14} />}
            onClick={() => handleOpenModal()}
          >
            {t("common.new", { defaultValue: "新建" })}
          </Button>
        </div>
      </div>

      {/* Table */}
      <div className="flex-1 overflow-auto px-6 py-4">
        <Table
          dataSource={optimisticClients ?? clientsQuery.data ?? []}
          columns={columns}
          rowKey="id"
          loading={clientsQuery.isLoading}
          pagination={false}
          onReorder={handleReorder}
          sortDisabled={reorderMutation.isPending}
        />
      </div>
    </div>
  );
}
