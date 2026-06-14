import {
  AppSidebar,
  Button,
  DeleteOutlined,
  Form,
  Input,
  ScrollArea,
  SegmentedControl,
  Select,
} from "@tokimo/ui";
import {
  CloudDownload,
  Cog,
  Globe,
  HardDrive,
  Monitor,
  Radio,
  Satellite,
  Zap,
} from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type DownloadClientDto,
  type DownloadClientType,
  downloadsApi,
} from "../api/client";

// ── Type config ──────────────────────────────────────────────────────────────

type ClientTypeConfig = {
  type: DownloadClientType;
  label: string;
  icon: typeof HardDrive;
  urlPlaceholder: string;
  experimental?: boolean;
};

const CLIENT_TYPE_LIST: ClientTypeConfig[] = [
  {
    type: "qbittorrent",
    label: "qBittorrent",
    icon: CloudDownload,
    urlPlaceholder: "http://localhost:8080",
  },
  {
    type: "transmission",
    label: "Transmission",
    icon: Radio,
    urlPlaceholder: "http://localhost:9091",
  },
  {
    type: "aria2",
    label: "Aria2",
    icon: Zap,
    urlPlaceholder: "http://localhost:6800",
  },
  {
    type: "deluge",
    label: "Deluge",
    icon: CloudDownload,
    urlPlaceholder: "http://localhost:8112",
  },
  {
    type: "rtorrent",
    label: "rTorrent",
    icon: Satellite,
    urlPlaceholder: "http://localhost:80",
  },
  {
    type: "synology",
    label: "Synology Download Station",
    icon: Cog,
    urlPlaceholder: "http://192.168.1.1:5000",
  },
  {
    type: "xunlei",
    label: "迅雷远程下载",
    icon: Globe,
    urlPlaceholder: "http://localhost:2345",
    experimental: true,
  },
  {
    type: "pan115",
    label: "115 网盘",
    icon: Monitor,
    urlPlaceholder: "http://localhost:10800",
    experimental: true,
  },
];

// ── Download paths field ─────────────────────────────────────────────────────

const CATEGORY_PATH_OPTIONS = [
  { label: "全局", value: "global" },
  { label: "电影", value: "movie" },
  { label: "剧集", value: "tv" },
  { label: "动漫", value: "anime" },
  { label: "纪录片", value: "documentary" },
  { label: "综艺", value: "variety" },
  { label: "体育", value: "sports" },
  { label: "MV", value: "mv" },
  { label: "音乐", value: "music" },
  { label: "电子书", value: "ebook" },
  { label: "有声书", value: "audiobook" },
  { label: "软件", value: "software" },
  { label: "游戏", value: "game" },
  { label: "课程", value: "course" },
  { label: "其他", value: "other" },
];

interface DownloadPathRow {
  type: string;
  path: string;
  description: string;
}

function DownloadPathsField({
  value,
  onChange,
}: {
  value: DownloadPathRow[];
  onChange: (v: DownloadPathRow[]) => void;
}) {
  const usedTypes = new Set(value.map((r) => r.type));
  const availableOptions = CATEGORY_PATH_OPTIONS.filter(
    (o) => !usedTypes.has(o.value),
  );

  const addRow = () => {
    const nextType = availableOptions[0]?.value ?? "other";
    onChange([...value, { type: nextType, path: "", description: "" }]);
  };

  const removeRow = (idx: number) => {
    if (value[idx]?.type === "global") return;
    onChange(value.filter((_, i) => i !== idx));
  };

  const updateRow = (
    idx: number,
    field: keyof DownloadPathRow,
    val: string,
  ) => {
    const next = [...value];
    next[idx] = { ...next[idx], [field]: val };
    onChange(next);
  };

  const globalPath = value.find((r) => r.type === "global")?.path ?? "";

  return (
    <div className="border-t border-black/[0.06] dark:border-white/[0.08] pt-4 mt-2">
      <p className="text-sm font-semibold mb-1">下载路径</p>
      <p className="text-xs text-fg-muted mb-3">
        按分类设置下载路径。全局路径必填，子分类留空则自动拼接为
        全局路径/分类名。
      </p>

      <div className="space-y-2">
        <div className="grid grid-cols-[100px_1fr_140px_32px] gap-2 text-xs text-fg-muted">
          <span>类型</span>
          <span>路径</span>
          <span>备注</span>
          <span />
        </div>

        {value.map((row, idx) => {
          const isGlobal = row.type === "global";
          const computedPath =
            !row.path && !isGlobal && globalPath
              ? `${globalPath.replace(/\/$/, "")}/${row.type}`
              : undefined;

          return (
            <div
              key={row.type || idx}
              className="grid grid-cols-[100px_1fr_140px_32px] gap-2 items-center"
            >
              <Select
                options={CATEGORY_PATH_OPTIONS.filter(
                  (o) => o.value === row.type || !usedTypes.has(o.value),
                )}
                value={row.type}
                onChange={(val: string) => updateRow(idx, "type", val)}
              />
              <Input
                placeholder={
                  isGlobal
                    ? "必填，如 /downloads"
                    : computedPath
                      ? computedPath
                      : "可选，留空自动拼接"
                }
                value={row.path}
                onChange={(e) => updateRow(idx, "path", e.target.value)}
              />
              <Input
                placeholder="备注（可选）"
                value={row.description}
                onChange={(e) => updateRow(idx, "description", e.target.value)}
              />
              <Button
                variant="text"
                danger
                icon={<DeleteOutlined />}
                disabled={isGlobal}
                onClick={() => removeRow(idx)}
                className="shrink-0"
              />
            </div>
          );
        })}
      </div>

      {availableOptions.length > 0 && (
        <Button variant="text" size="small" onClick={addRow} className="mt-2">
          + 添加下载路径
        </Button>
      )}
    </div>
  );
}

// ── Window component ─────────────────────────────────────────────────────────

interface WindowHandle {
  id: string;
  metadata: Record<string, unknown>;
  close: () => void;
}

interface Props {
  win: WindowHandle;
}

export default function ClientFormWindow({ win }: Props) {
  const { t } = useTranslation();
  const [form] = Form.useForm();
  const dc = "media.downloadClients";

  const editingClient = win.metadata.editingClient as
    | DownloadClientDto
    | undefined;
  const onSaved = win.metadata.onSaved as (() => void) | undefined;

  const [submitting, setSubmitting] = useState(false);
  const [downloadPaths, setDownloadPaths] = useState<DownloadPathRow[]>(() => {
    if (editingClient?.downloadPaths?.length) {
      return editingClient.downloadPaths.map((p, i) => ({
        type: (p as Record<string, string>).type ?? (i === 0 ? "global" : ""),
        path: p.path,
        description: p.description,
      }));
    }
    return [{ type: "global", path: "", description: "" }];
  });

  // Initialize form values
  useState(() => {
    if (editingClient) {
      form.setFieldsValue({
        name: editingClient.name,
        type: editingClient.type,
        url: editingClient.url,
        username: editingClient.username,
        password: editingClient.password,
        requireAuth: editingClient.requireAuth,
        monitorEnabled: editingClient.monitorEnabled,
      });
    } else {
      form.setFieldsValue({
        type: "qbittorrent",
        requireAuth: true,
      });
    }
  });

  const selectedType = (Form.useWatch("type", form) ??
    "qbittorrent") as DownloadClientType;
  const selectedTypeCfg =
    CLIENT_TYPE_LIST.find((c) => c.type === selectedType) ??
    CLIENT_TYPE_LIST[0];

  const sidebarSections = useMemo(
    () => [
      {
        key: "client-types",
        label: t(`${dc}.type`),
        items: CLIENT_TYPE_LIST.map((cfg) => ({
          key: cfg.type,
          icon: <cfg.icon className="h-4 w-4" />,
          label: cfg.label,
          extra: cfg.experimental ? "Beta" : undefined,
        })),
      },
    ],
    [t],
  );

  const handleSubmit = async () => {
    const values = await form.validateFields();
    const globalRow = downloadPaths.find((p) => p.type === "global");
    if (!globalRow?.path.trim()) return;
    const validPaths = downloadPaths.filter((p) => p.type);
    setSubmitting(true);
    try {
      if (editingClient) {
        await downloadsApi.clients.update(editingClient.id, {
          ...values,
          downloadPaths: validPaths,
        });
      } else {
        await downloadsApi.clients.create({
          ...values,
          downloadPaths: validPaths,
        });
      }
      // Notify parent to refresh, then close
      onSaved?.();
      win.close();
    } catch {
      // errors handled by API
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex flex-1 min-h-0 overflow-hidden">
        <AppSidebar
          sections={sidebarSections}
          activeKey={selectedType}
          onSelect={(key) =>
            form.setFieldValue("type", key as DownloadClientType)
          }
        />

        <ScrollArea
          direction="vertical"
          className="flex-1 min-h-0"
          innerClassName="px-6 py-5"
        >
          <Form form={form} layout="vertical" autoComplete="off">
            <Form.Item name="type" hidden>
              <Input />
            </Form.Item>

            <Form.Item
              name="name"
              label={t(`${dc}.name`)}
              rules={[{ required: true, message: t(`${dc}.nameRequired`) }]}
              extra={t(`${dc}.nameExtra`)}
            >
              <Input placeholder={t(`${dc}.namePlaceholder`)} />
            </Form.Item>

            <Form.Item
              name="url"
              label={t(`${dc}.accessUrl`)}
              rules={[
                { required: true, message: t(`${dc}.urlRequired`) },
                { type: "url", message: t(`${dc}.urlInvalid`) },
              ]}
              extra={t(`${dc}.urlExtra`)}
            >
              <Input
                placeholder={
                  selectedTypeCfg?.urlPlaceholder ?? "http://localhost:8080"
                }
              />
            </Form.Item>

            <div className="grid grid-cols-2 gap-x-4">
              <Form.Item
                name="username"
                label={t(`${dc}.loginUsername`)}
                extra={t(`${dc}.loginUsernameExtra`)}
              >
                <Input placeholder="admin" autoComplete="new-password" />
              </Form.Item>

              <Form.Item
                name="password"
                label={t(`${dc}.loginPassword`)}
                extra={t(`${dc}.loginPasswordExtra`)}
              >
                <Input.Password
                  placeholder={t(`${dc}.passwordPlaceholder`)}
                  autoComplete="new-password"
                />
              </Form.Item>
            </div>

            <div className="grid grid-cols-2 gap-x-4">
              <Form.Item
                name="requireAuth"
                label={t(`${dc}.requireAuth`)}
                extra={t(`${dc}.requireAuthExtra`)}
              >
                <SegmentedControl<boolean>
                  options={[
                    { label: t("media.common.enable"), value: true },
                    { label: t("media.common.disable"), value: false },
                  ]}
                />
              </Form.Item>

              <Form.Item
                name="monitorEnabled"
                label={t(`${dc}.monitorEnabled`)}
                extra={t(`${dc}.monitorEnabledExtra`)}
              >
                <SegmentedControl<boolean>
                  options={[
                    { label: t("media.common.enable"), value: true },
                    { label: t("media.common.disable"), value: false },
                  ]}
                />
              </Form.Item>
            </div>

            <DownloadPathsField
              value={downloadPaths}
              onChange={setDownloadPaths}
            />
          </Form>
        </ScrollArea>
      </div>

      <div className="flex justify-end gap-2 px-6 py-4 border-t border-black/[0.06] dark:border-white/[0.08] shrink-0">
        <Button onClick={win.close}>{t("media.common.cancel")}</Button>
        <Button
          variant="primary"
          loading={submitting}
          onClick={() => void handleSubmit()}
        >
          {t("media.common.confirm")}
        </Button>
      </div>
    </div>
  );
}
