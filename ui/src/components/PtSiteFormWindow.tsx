import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AppSidebar,
  Button,
  Form,
  Input,
  ScrollArea,
  Select,
} from "@tokimo/ui";
import { Globe } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ptSitesApi } from "../api/client";

interface WindowHandle {
  id: string;
  metadata: Record<string, unknown>;
  close: () => void;
}

interface Props {
  win: WindowHandle;
}

export default function PtSiteFormWindow({ win }: Props) {
  const { t } = useTranslation();
  const [form] = Form.useForm();
  const utils = useQueryClient();

  const editingSiteId = win.metadata.editingSiteId as string | undefined;
  const onSaved = win.metadata.onSaved as (() => void) | undefined;

  const [submitting, setSubmitting] = useState(false);
  const initialized = useRef(false);

  const availableQuery = useQuery({
    queryKey: ["pt-sites", "available"],
    queryFn: () => ptSitesApi.getAvailableSites(),
  });

  const editingSiteQuery = useQuery({
    queryKey: ["pt-sites", editingSiteId],
    queryFn: () => ptSitesApi.getById(editingSiteId!),
    enabled: !!editingSiteId,
  });

  const availableSites = availableQuery.data ?? [];
  const editingSite = editingSiteQuery.data;
  const selectedSiteId = (Form.useWatch("siteId", form) as string) || "";

  // Initialize form with editing site or first available site
  useEffect(() => {
    if (initialized.current) return;

    if (editingSiteId && editingSite) {
      initialized.current = true;
      form.setFieldsValue({
        siteId: editingSite.siteId,
        name: editingSite.name,
        domain: editingSite.domain ?? "",
        authType: editingSite.authType ?? "cookie",
      });
    } else if (!editingSiteId && availableSites.length > 0) {
      initialized.current = true;
      const first = availableSites[0];
      form.setFieldsValue({
        siteId: first.id,
        name: first.name,
        domain: first.domain,
        authType: first.allowAuthType?.[0] ?? "cookie",
      });
    }
  }, [editingSiteId, editingSite, availableSites, form]);

  const selectedSite = availableSites.find((s) => s.id === selectedSiteId);

  const sidebarSections = useMemo(
    () => [
      {
        key: "sites",
        label: "站点",
        items: availableSites.map((site) => ({
          key: site.id,
          icon: <Globe className="h-4 w-4" />,
          label: site.name,
        })),
      },
    ],
    [availableSites],
  );

  const handleSubmit = async () => {
    const values = await form.validateFields();
    setSubmitting(true);
    try {
      if (editingSite) {
        await ptSitesApi.update(editingSite.id, {
          name: values.name,
          siteId: values.siteId,
          domain: values.domain || undefined,
          authType: values.authType || "cookie",
          cookies: values.cookies || undefined,
          apiKey: values.apiKey || undefined,
        });
      } else {
        await ptSitesApi.create({
          name: values.name,
          siteId: values.siteId,
          domain: values.domain || undefined,
          authType: values.authType || "cookie",
          cookies: values.cookies || undefined,
          apiKey: values.apiKey || undefined,
        });
      }
      await utils.invalidateQueries({ queryKey: ["pt-sites"] });
      onSaved?.();
      win.close();
    } catch {
      // error
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="relative flex flex-1 min-h-0 overflow-hidden">
        <AppSidebar
          sections={sidebarSections}
          activeKey={selectedSiteId}
          onSelect={(key) => {
            const site = availableSites.find((s) => s.id === key);
            if (site) {
              form.setFieldsValue({
                siteId: key,
                name: site.name,
                domain: site.domain,
                authType: site.allowAuthType?.[0] ?? "cookie",
              });
            }
          }}
        />

        <ScrollArea
          direction="vertical"
          className="flex-1 min-h-0"
          innerClassName="px-6 py-5"
        >
          <Form form={form} layout="vertical" autoComplete="off">
            <Form.Item name="siteId" hidden>
              <Input />
            </Form.Item>

            <Form.Item
              name="name"
              label="站点名称"
              rules={[{ required: true, message: "请输入站点名称" }]}
            >
              <Input placeholder="例如：HDHome" />
            </Form.Item>

            <Form.Item name="domain" label="域名">
              <Input placeholder={selectedSite?.domain ?? "自动检测"} />
            </Form.Item>

            <Form.Item name="authType" label="认证方式" initialValue="cookie">
              <Select
                options={[
                  { label: "Cookie", value: "cookie" },
                  { label: "API Key", value: "api_key" },
                ]}
              />
            </Form.Item>

            <Form.Item
              noStyle
              shouldUpdate={(prev, cur) => prev.authType !== cur.authType}
            >
              {({ getFieldValue }) =>
                getFieldValue("authType") === "api_key" ? (
                  <Form.Item name="apiKey" label="API Key">
                    <Input.Password placeholder="输入 API Key" />
                  </Form.Item>
                ) : (
                  <Form.Item name="cookies" label="Cookie">
                    <Input.TextArea
                      rows={4}
                      placeholder="从浏览器复制 Cookie..."
                    />
                  </Form.Item>
                )
              }
            </Form.Item>
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
