import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useWindowActions } from "@tokimo/sdk";
import {
  Button,
  DeleteOutlined,
  EditOutlined,
  Modal,
  PauseCircleOutlined,
  PlayCircleOutlined,
  ReloadOutlined,
  Table,
  Tag,
  useDateFormat,
} from "@tokimo/ui";
import { FileText, PlayIcon, Plus } from "lucide-react";
import { useCallback } from "react";
import { type Subscription, subscriptionsApi } from "../api/client";

const statusColors: Record<string, string> = {
  active: "blue",
  paused: "orange",
  pushed: "green",
  completed: "green",
  expired: "default",
};

const statusLabels: Record<string, string> = {
  active: "运行中",
  paused: "已暂停",
  pushed: "已推送下载",
  completed: "已完成",
  expired: "已过期",
};

const mediaTypeLabels: Record<Subscription["mediaType"], string> = {
  movie: "电影",
  tv: "剧集",
};

export default function SubscriptionsPage() {
  const utils = useQueryClient();
  const windowActions = useWindowActions();
  const { formatLong } = useDateFormat();

  const subsQuery = useQuery({
    queryKey: ["subscriptions"],
    queryFn: () => subscriptionsApi.list(),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => subscriptionsApi.delete(id),
    onSuccess: () => {
      void utils.invalidateQueries({ queryKey: ["subscriptions"] });
    },
  });

  const handleFormSaved = useCallback(() => {
    void utils.invalidateQueries({ queryKey: ["subscriptions"] });
  }, [utils]);

  const handleCreate = () => {
    windowActions.openModalWindow({
      component: () => import("./SubscriptionFormWindow"),
      title: "新建订阅",
      width: 680,
      height: 700,
      metadata: {
        editingSubscription: null,
        onSaved: handleFormSaved,
      },
    });
  };

  const handleEdit = (sub: Subscription) => {
    windowActions.openModalWindow({
      component: () => import("./SubscriptionFormWindow"),
      title: "编辑订阅",
      width: 680,
      height: 700,
      metadata: {
        editingSubscription: sub,
        onSaved: handleFormSaved,
      },
    });
  };

  const handleDelete = (sub: Subscription) => {
    Modal.confirm({
      title: "确认删除",
      content: `确定要删除订阅「${sub.title}」吗？`,
      okText: "删除",
      variant: "danger",
      cancelText: "取消",
      onOk: () => deleteMutation.mutateAsync(sub.id),
    });
  };

  const handleTogglePause = (sub: Subscription) => {
    const newStatus = sub.status === "active" ? "paused" : "active";
    subscriptionsApi.update(sub.id, { status: newStatus }).then(() => {
      void utils.invalidateQueries({ queryKey: ["subscriptions"] });
    });
  };

  const handleExecute = (sub: Subscription) => {
    subscriptionsApi.execute(sub.id).then(() => {
      void utils.invalidateQueries({ queryKey: ["subscriptions"] });
    });
  };

  const handleViewLogs = (sub: Subscription) => {
    windowActions.openModalWindow({
      component: () => import("./SubscriptionLogsWindow"),
      title: `执行日志 — ${sub.title}`,
      width: 800,
      height: 550,
      metadata: {
        subscriptionId: sub.id,
        subscriptionTitle: sub.title,
      },
    });
  };

  const columns = [
    {
      title: "",
      key: "poster",
      width: 48,
      render: (_: unknown, record: Subscription) =>
        record.posterPath ? (
          <img
            src={record.posterPath}
            alt={record.title}
            className="w-8 h-11 rounded object-cover"
          />
        ) : (
          <div className="w-8 h-11 rounded bg-muted" />
        ),
    },
    {
      title: "标题",
      key: "title",
      render: (_: unknown, record: Subscription) => (
        <div className="flex flex-col">
          <span className="font-medium">{record.title}</span>
          <span className="text-xs text-muted-foreground">
            {mediaTypeLabels[record.mediaType]}
            {record.season ? ` S${record.season}` : ""}
            {record.year ? ` (${record.year})` : ""}
          </span>
        </div>
      ),
    },
    {
      title: "状态",
      key: "status",
      width: 80,
      render: (_: unknown, record: Subscription) => (
        <Tag
          color={
            statusColors[record.status] as
              | "blue"
              | "orange"
              | "green"
              | "default"
          }
        >
          {statusLabels[record.status] ?? record.status}
        </Tag>
      ),
    },
    {
      title: "分类",
      key: "category",
      width: 70,
      render: (_: unknown, record: Subscription) =>
        record.category ? <Tag>{record.category}</Tag> : "-",
    },
    {
      title: "过滤",
      key: "filters",
      width: 180,
      render: (_: unknown, record: Subscription) => {
        const tags: string[] = [];
        if (record.sources?.length) tags.push(...record.sources);
        if (record.resolutions?.length) tags.push(...record.resolutions);
        if (record.codecs?.length) tags.push(...record.codecs);
        if (record.freeOnly) tags.push("免费");
        if (tags.length === 0)
          return <span className="text-muted-foreground">-</span>;
        return (
          <div className="flex flex-wrap gap-1">
            {tags.slice(0, 4).map((t) => (
              <Tag key={t} className="text-xs">
                {t}
              </Tag>
            ))}
            {tags.length > 4 && (
              <span className="text-xs text-muted-foreground">
                +{tags.length - 4}
              </span>
            )}
          </div>
        );
      },
    },
    {
      title: "间隔",
      key: "interval",
      width: 60,
      render: (_: unknown, record: Subscription) =>
        `${record.intervalMinutes}分`,
    },
    {
      title: "上次检查",
      key: "lastChecked",
      width: 180,
      render: (_: unknown, record: Subscription) =>
        record.lastCheckedAt ? formatLong(new Date(record.lastCheckedAt)) : "-",
    },
    {
      title: "操作",
      key: "actions",
      width: 170,
      render: (_: unknown, record: Subscription) => {
        const isTerminal = record.status === "pushed";
        return (
          <div className="flex items-center gap-1">
            {!isTerminal && (
              <Button
                variant="text"
                icon={
                  record.status === "active" ? (
                    <PauseCircleOutlined />
                  ) : (
                    <PlayCircleOutlined />
                  )
                }
                onClick={() => handleTogglePause(record)}
                title={record.status === "active" ? "暂停" : "恢复"}
              />
            )}
            {!isTerminal && (
              <Button
                variant="text"
                icon={<PlayIcon size={14} />}
                onClick={() => handleExecute(record)}
                title="手动执行"
              />
            )}
            <Button
              variant="text"
              icon={<FileText size={14} />}
              onClick={() => handleViewLogs(record)}
              title="查看日志"
            />
            <Button
              variant="text"
              icon={<EditOutlined />}
              onClick={() => handleEdit(record)}
            />
            <Button
              variant="text"
              danger
              icon={<DeleteOutlined />}
              onClick={() => handleDelete(record)}
            />
          </div>
        );
      },
    },
  ];

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-lg font-semibold">订阅管理</h1>
        <div className="flex gap-2">
          <Button
            icon={<ReloadOutlined />}
            onClick={() => void subsQuery.refetch()}
            loading={subsQuery.isRefetching}
          />
          <Button
            variant="primary"
            icon={<Plus size={14} />}
            onClick={handleCreate}
          >
            新建订阅
          </Button>
        </div>
      </div>

      <Table
        dataSource={subsQuery.data ?? []}
        columns={columns}
        rowKey="id"
        loading={subsQuery.isLoading}
        pagination={false}
      />
    </div>
  );
}
