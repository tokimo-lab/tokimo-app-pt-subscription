import { useQueryClient } from "@tanstack/react-query";
import {
  AppSidebar,
  Button,
  DeleteOutlined,
  Form,
  Input,
  ScrollArea,
  SegmentedControl,
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

interface DownloadPathRow {
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
  const { t } = useTranslation();
  const dc = "media.downloadClients";

  const addRow = () => onChange([...value, { path: "", description: "" }]);
  const removeRow = (idx: number) => {
    if (value.length <= 1) return; // must keep at least one
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

  return (
    <div className="border-t border-black/[0.06] dark:border-white/[0.08] pt-4 mt-2">
      <p className="text-sm font-semibold mb-1">{t(`${dc}.downloadPaths`)}</p>
      <p className="text-xs text-fg-muted mb-3">
        {t(`${dc}.downloadPathsHint`)}
      </p>

      <div className="space-y-2">
        {value.map((row, idx) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: dynamic list without stable IDs
          <div key={idx} className="flex items-start gap-2">
            <Input
              className="flex-1"
              placeholder={t(`${dc}.downloadPathPlaceholder`)}
              value={row.path}
              onChange={(e) => updateRow(idx, "path", e.target.value)}
            />
            <Input
              className="flex-1"
              placeholder={t(`${dc}.downloadPathDescPlaceholder`)}
              value={row.description}
              onChange={(e) => updateRow(idx, "description", e.target.value)}
            />
            <Button
              variant="text"
              danger
              icon={<DeleteOutlined />}
              disabled={value.length <= 1}
              onClick={() => removeRow(idx)}
              className="shrink-0"
            />
          </div>
        ))}
      </div>

      <Button variant="text" size="small" onClick={addRow} className="mt-2">
        + {t(`${dc}.addDownloadPath`)}
      </Button>
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
  const utils = useQueryClient();
  const dc = "media.downloadClients";

  const editingClient = win.metadata.editingClient as
    | DownloadClientDto
    | undefined;

  const [submitting, setSubmitting] = useState(false);
  const [downloadPaths, setDownloadPaths] = useState<DownloadPathRow[]>(() => {
    if (editingClient?.downloadPaths?.length) {
      return editingClient.downloadPaths;
    }
    return [{ path: "", description: "" }];
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

  // ── Sidebar sections ──
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
    // Validate download paths
    const validPaths = downloadPaths.filter((p) => p.path.trim());
    if (validPaths.length === 0) {
      return;
    }
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
      void utils.invalidateQueries({ queryKey: ["downloads", "clients"] });
      win.close();
    } catch {
      // errors handled by mutation
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* ── Body: type selector + form ── */}
      <div className="flex flex-1 min-h-0 overflow-hidden">
        {/* ── Left: AppSidebar type selector ── */}
        <AppSidebar
          sections={sidebarSections}
          activeKey={selectedType}
          onSelect={(key) =>
            form.setFieldValue("type", key as DownloadClientType)
          }
        />

        {/* ── Right: form ── */}
        <ScrollArea
          direction="vertical"
          className="flex-1 min-h-0"
          innerClassName="px-6 py-5"
        >
          <Form form={form} layout="vertical" autoComplete="off">
            {/* hidden type field */}
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

      {/* ── Footer ── */}
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
