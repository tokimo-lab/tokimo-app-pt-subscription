import { useQuery } from "@tanstack/react-query";
import {
  AppSidebar,
  Button,
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
  categoriesApi,
  type DownloadClientDto,
  type DownloadClientType,
  downloadsApi,
} from "../api/client";
import { categoryLabel } from "./search-utils";

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
  type: string;
  path: string;
  description: string;
}

const dc = "media.downloadClients";

function DownloadPathsField({
  value,
  onChange,
}: {
  value: DownloadPathRow[];
  onChange: (v: DownloadPathRow[]) => void;
}) {
  const { t } = useTranslation();
  const categoriesQuery = useQuery({
    queryKey: ["categories"],
    queryFn: () => categoriesApi.list(),
  });

  const slugs = useMemo(
    () => [
      "global",
      ...(categoriesQuery.data?.categories ?? []).map((c) => c.slug),
    ],
    [categoriesQuery.data],
  );

  const rowBySlug = useMemo(() => {
    const map = new Map<string, DownloadPathRow>();
    for (const row of value) map.set(row.type, row);
    return map;
  }, [value]);

  const globalPath = rowBySlug.get("global")?.path ?? "";

  const updateRow = (
    slug: string,
    field: "path" | "description",
    val: string,
  ) => {
    const existing = rowBySlug.get(slug);
    if (existing) {
      onChange(
        value.map((r) => (r.type === slug ? { ...r, [field]: val } : r)),
      );
    } else {
      onChange([
        ...value,
        { type: slug, path: "", description: "", [field]: val },
      ]);
    }
  };

  return (
    <div className="border-t border-black/[0.06] dark:border-white/[0.08] pt-4 mt-2">
      <p className="text-sm font-semibold mb-1">{t(`${dc}.downloadPaths`)}</p>
      <p className="text-xs text-fg-muted mb-3">
        {t(`${dc}.downloadPathsHint`)}
      </p>

      <div className="space-y-2">
        <div className="grid grid-cols-[110px_1fr_160px] gap-2 text-xs text-fg-muted">
          <span>{t(`${dc}.downloadPathColCategory`)}</span>
          <span>{t(`${dc}.downloadPathColPath`)}</span>
          <span>{t(`${dc}.downloadPathColDesc`)}</span>
        </div>

        {slugs.map((slug) => {
          const isGlobal = slug === "global";
          const row = rowBySlug.get(slug);
          const path = row?.path ?? "";
          const description = row?.description ?? "";
          const computedPath =
            !path && !isGlobal && globalPath
              ? `${globalPath.replace(/\/$/, "")}/${slug}`
              : undefined;

          return (
            <div
              key={slug}
              className="grid grid-cols-[110px_1fr_160px] gap-2 items-center"
            >
              <span className="text-sm truncate">
                {isGlobal ? t("category.global") : categoryLabel(slug, t)}
                {isGlobal && <span className="text-red-500 ml-0.5">*</span>}
              </span>
              <Input
                placeholder={
                  isGlobal
                    ? t(`${dc}.globalPathPlaceholder`)
                    : (computedPath ?? t(`${dc}.subPathPlaceholder`))
                }
                value={path}
                onChange={(e) => updateRow(slug, "path", e.target.value)}
              />
              <Input
                placeholder={t(`${dc}.downloadPathDescPlaceholder`)}
                value={description}
                onChange={(e) => updateRow(slug, "description", e.target.value)}
              />
            </div>
          );
        })}
      </div>
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

  const editingClient = win.metadata.editingClient as
    | DownloadClientDto
    | undefined;
  const onSaved = win.metadata.onSaved as (() => void) | undefined;

  const [submitting, setSubmitting] = useState(false);
  const [downloadPaths, setDownloadPaths] = useState<DownloadPathRow[]>(() => {
    const rows: DownloadPathRow[] = (editingClient?.downloadPaths ?? [])
      .map((p) => ({
        type: (p as Record<string, string>).type ?? "",
        path: p.path,
        description: p.description,
      }))
      .filter((r) => r.type);
    if (!rows.some((r) => r.type === "global")) {
      rows.unshift({ type: "global", path: "", description: "" });
    }
    return rows;
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
    const validPaths = downloadPaths.filter(
      (p) => p.type === "global" || p.path.trim(),
    );
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
