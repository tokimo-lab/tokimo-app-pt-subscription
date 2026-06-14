import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useWindowActions } from "@tokimo/sdk";
import {
  Button,
  DeleteOutlined,
  EditOutlined,
  Modal,
  ReloadOutlined,
  Table,
  Tooltip,
} from "@tokimo/ui";
import { Plus } from "lucide-react";
import { useCallback } from "react";
import {
  type PtSiteDto,
  type PtSiteStatusDto,
  ptSitesApi,
} from "../api/client";

function LoginStatusCell({
  status,
  isLoading,
}: {
  status: PtSiteStatusDto | undefined;
  isLoading: boolean;
}) {
  if (isLoading) {
    return (
      <span className="flex items-center gap-1.5">
        <span className="inline-block h-2 w-2 animate-pulse rounded-full bg-gray-400" />
        <span className="text-fg-muted text-sm">检测中...</span>
      </span>
    );
  }

  if (!status) {
    return (
      <span className="flex items-center gap-1.5">
        <span className="inline-block h-2 w-2 rounded-full bg-red-500" />
        <span className="text-sm text-red-600 dark:text-red-400">未知</span>
      </span>
    );
  }

  const statusNode = (
    <span className="flex items-center gap-1.5">
      <span
        className={`inline-block h-2 w-2 rounded-full ${
          status.isLoggedIn ? "bg-green-500" : "bg-red-500"
        }`}
      />
      <span
        className={`text-sm ${
          status.isLoggedIn
            ? "text-green-600 dark:text-green-400"
            : "text-red-600 dark:text-red-400"
        }`}
      >
        {status.isLoggedIn ? "已登录" : "未登录"}
      </span>
    </span>
  );

  if (!status.isLoggedIn && status.errorMessage) {
    return <Tooltip title={status.errorMessage}>{statusNode}</Tooltip>;
  }

  return statusNode;
}

export default function PtSitesPage() {
  const utils = useQueryClient();
  const windowActions = useWindowActions();

  const sitesQuery = useQuery({
    queryKey: ["pt-sites"],
    queryFn: () => ptSitesApi.list(),
  });

  const statusQuery = useQuery({
    queryKey: ["pt-sites", "status"],
    queryFn: () => ptSitesApi.allStatus(),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => ptSitesApi.delete(id),
    onSuccess: () => {
      void utils.invalidateQueries({ queryKey: ["pt-sites"] });
    },
  });

  const handleFormSaved = useCallback(() => {
    void utils.invalidateQueries({ queryKey: ["pt-sites"] });
  }, [utils]);

  const handleAddSite = () => {
    windowActions.openModalWindow({
      component: () => import("./PtSiteFormWindow"),
      title: "添加 PT 站点",
      width: 760,
      height: 600,
      metadata: { onSaved: handleFormSaved },
    });
  };

  const handleEditSite = (site: PtSiteDto) => {
    windowActions.openModalWindow({
      component: () => import("./PtSiteFormWindow"),
      title: `编辑 — ${site.name}`,
      width: 760,
      height: 600,
      metadata: { editingSiteId: site.id, onSaved: handleFormSaved },
    });
  };

  const handleDelete = (site: PtSiteDto) => {
    Modal.confirm({
      title: "确认删除",
      content: `确定要删除 PT 站点「${site.name}」吗？`,
      okText: "删除",
      variant: "danger",
      cancelText: "取消",
      onOk: () => deleteMutation.mutateAsync(site.id),
    });
  };

  const statusMap = new Map((statusQuery.data ?? []).map((s) => [s.id, s]));

  const columns = [
    {
      title: "名称",
      dataIndex: "name",
      key: "name",
      render: (name: string) => <span className="font-medium">{name}</span>,
    },
    {
      title: "登录状态",
      key: "loginStatus",
      render: (_: unknown, record: PtSiteDto) => (
        <LoginStatusCell
          status={statusMap.get(record.id)}
          isLoading={statusQuery.isLoading}
        />
      ),
    },
    {
      title: "用户",
      key: "username",
      render: (_: unknown, record: PtSiteDto) =>
        statusMap.get(record.id)?.userInfo?.username ?? "-",
    },
    {
      title: "分享率",
      key: "shareRatio",
      render: (_: unknown, record: PtSiteDto) =>
        statusMap.get(record.id)?.userInfo?.shareRatio ?? "-",
    },
    {
      title: "操作",
      key: "actions",
      width: 120,
      render: (_: unknown, record: PtSiteDto) => (
        <div className="flex items-center gap-1">
          <Button
            variant="text"
            icon={<EditOutlined />}
            onClick={() => handleEditSite(record)}
            title="编辑"
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

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-lg font-semibold">PT 站点</h1>
        <div className="flex gap-2">
          <Button
            icon={<ReloadOutlined />}
            onClick={() => {
              void sitesQuery.refetch();
              void statusQuery.refetch();
            }}
            loading={sitesQuery.isRefetching || statusQuery.isRefetching}
          />
          <Button
            variant="primary"
            icon={<Plus size={14} />}
            onClick={handleAddSite}
          >
            添加站点
          </Button>
        </div>
      </div>

      <Table
        dataSource={sitesQuery.data ?? []}
        columns={columns}
        rowKey="id"
        loading={sitesQuery.isLoading}
        pagination={false}
      />
    </div>
  );
}
