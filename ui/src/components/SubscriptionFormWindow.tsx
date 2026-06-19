import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Checkbox, Form, Input, ScrollArea, Select } from "@tokimo/ui";
import { ChevronDown, ChevronRight } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type CreateSubscriptionInput,
  categoriesApi,
  downloadsApi,
  filterOptionsApi,
  ptSitesApi,
  type Subscription,
  subscriptionsApi,
} from "../api/client";
import { categoryLabel } from "./search-utils";

interface WindowHandle {
  id: string;
  metadata: Record<string, unknown>;
  close: () => void;
}

interface Props {
  win: WindowHandle;
}

export default function SubscriptionFormWindow({ win }: Props) {
  const [form] = Form.useForm();
  const utils = useQueryClient();
  const { t } = useTranslation();

  const editingSub = win.metadata.editingSubscription as Subscription | null;
  const onSaved = win.metadata.onSaved as (() => void) | undefined;
  const prefilledData = win.metadata.prefilled as
    | Partial<CreateSubscriptionInput>
    | undefined;

  const [submitting, setSubmitting] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const initialized = useRef(false);

  const categoriesQuery = useQuery({
    queryKey: ["categories"],
    queryFn: () => categoriesApi.list(),
  });

  const filterOptionsQuery = useQuery({
    queryKey: ["filter-options"],
    queryFn: () => filterOptionsApi.list(),
  });

  const clientsQuery = useQuery({
    queryKey: ["downloads", "clients"],
    queryFn: () => downloadsApi.clients.list(),
  });

  const sitesQuery = useQuery({
    queryKey: ["pt-sites"],
    queryFn: () => ptSitesApi.list(),
  });

  const categories = categoriesQuery.data?.categories ?? [];
  const filterOptions = filterOptionsQuery.data;
  const clients = clientsQuery.data ?? [];
  const sites = sitesQuery.data ?? [];

  useEffect(() => {
    if (initialized.current) return;
    if (categoriesQuery.isLoading || clientsQuery.isLoading) return;

    if (editingSub) {
      initialized.current = true;
      form.setFieldsValue({
        mediaType: editingSub.mediaType,
        title: editingSub.title,
        tmdbId: editingSub.tmdbId ?? undefined,
        year: editingSub.year ?? undefined,
        posterPath: editingSub.posterPath ?? undefined,
        season: editingSub.season ?? undefined,
        episodes: editingSub.episodes?.join(", ") ?? undefined,
        category: editingSub.category ?? undefined,
        downloadClientId: editingSub.downloadClientId ?? undefined,
        siteIds: editingSub.siteIds ?? [],
        sources: editingSub.sources ?? [],
        resolutions: editingSub.resolutions ?? [],
        codecs: editingSub.codecs ?? [],
        includeKeywords: editingSub.includeKeywords ?? undefined,
        excludeKeywords: editingSub.excludeKeywords ?? undefined,
        freeOnly: editingSub.freeOnly,
        excludeHr: editingSub.excludeHr,
        releaseGroups: editingSub.releaseGroups?.join(", ") ?? undefined,
        minSize: editingSub.minSize || undefined,
        maxSize: editingSub.maxSize || undefined,
        minSeeders: editingSub.minSeeders || undefined,
        maxSeeders: editingSub.maxSeeders || undefined,
        intervalMinutes: editingSub.intervalMinutes,
      });
    } else if (prefilledData) {
      initialized.current = true;
      const posterUrl = prefilledData.posterPath
        ? `https://image.tmdb.org/t/p/w300${prefilledData.posterPath}`
        : undefined;
      const defaultClient = clients.find((c) => c.isDefault);
      form.setFieldsValue({
        mediaType: prefilledData.mediaType ?? "movie",
        title: prefilledData.title ?? "",
        tmdbId: prefilledData.tmdbId ?? undefined,
        year: prefilledData.year ?? undefined,
        posterPath: posterUrl,
        category: prefilledData.mediaType === "tv" ? "tv" : "movie",
        downloadClientId: defaultClient?.id,
      });
    } else {
      initialized.current = true;
      const defaultClient = clients.find((c) => c.isDefault);
      form.setFieldsValue({
        mediaType: "movie",
        downloadClientId: defaultClient?.id,
      });
    }
  }, [
    editingSub,
    prefilledData,
    categoriesQuery.isLoading,
    clientsQuery.isLoading,
    clients,
    form,
  ]);

  const mediaType = Form.useWatch("mediaType", form) as string;

  const handleSubmit = async () => {
    const values = await form.validateFields();
    setSubmitting(true);
    try {
      const episodesArr = values.episodes
        ? String(values.episodes)
            .split(",")
            .map((s: string) => parseInt(s.trim(), 10))
            .filter((n: number) => !Number.isNaN(n))
        : undefined;
      const releaseGroupsArr = values.releaseGroups
        ? String(values.releaseGroups)
            .split(",")
            .map((s: string) => s.trim())
            .filter(Boolean)
        : undefined;

      const input: CreateSubscriptionInput = {
        mediaType: values.mediaType,
        title: values.title,
        tmdbId: values.tmdbId ? Number(values.tmdbId) : undefined,
        year: values.year || undefined,
        posterPath: values.posterPath || undefined,
        season: values.season ? Number(values.season) : undefined,
        episodes: episodesArr,
        category: values.category || undefined,
        downloadClientId: values.downloadClientId || undefined,
        siteIds: values.siteIds?.length ? values.siteIds : undefined,
        sources: values.sources?.length ? values.sources : undefined,
        resolutions: values.resolutions?.length
          ? values.resolutions
          : undefined,
        codecs: values.codecs?.length ? values.codecs : undefined,
        includeKeywords: values.includeKeywords || undefined,
        excludeKeywords: values.excludeKeywords || undefined,
        freeOnly: values.freeOnly ?? false,
        excludeHr: values.excludeHr ?? false,
        releaseGroups: releaseGroupsArr,
        minSize: values.minSize ? Number(values.minSize) : undefined,
        maxSize: values.maxSize ? Number(values.maxSize) : undefined,
        minSeeders: values.minSeeders ? Number(values.minSeeders) : undefined,
        maxSeeders: values.maxSeeders ? Number(values.maxSeeders) : undefined,
        intervalMinutes: values.intervalMinutes
          ? Number(values.intervalMinutes)
          : 5,
      };

      if (editingSub) {
        await subscriptionsApi.update(editingSub.id, input);
      } else {
        await subscriptionsApi.create(input);
      }
      await utils.invalidateQueries({ queryKey: ["subscriptions"] });
      onSaved?.();
      win.close();
    } finally {
      setSubmitting(false);
    }
  };

  const categoryOptions = categories.map((c) => ({
    label: categoryLabel(c.slug, t),
    value: c.slug,
  }));

  const clientOptions = clients.map((c) => ({
    label: c.name,
    value: c.id,
  }));

  const siteOptions = sites.map((s) => ({
    label: s.name,
    value: s.id,
  }));

  return (
    <div className="flex flex-col h-full">
      <ScrollArea
        direction="vertical"
        className="flex-1 min-h-0"
        innerClassName="px-6 py-4"
      >
        <Form form={form} layout="vertical" autoComplete="off">
          {/* ── Row 1: 类型 + 标题 ── */}
          <div className="grid grid-cols-[120px_1fr] gap-3">
            <Form.Item name="mediaType" label="类型" initialValue="movie">
              <Select
                options={[
                  { label: "电影", value: "movie" },
                  { label: "剧集", value: "tv" },
                ]}
              />
            </Form.Item>
            <Form.Item
              name="title"
              label="标题"
              rules={[{ required: true, message: "请输入标题" }]}
            >
              <Input placeholder="影视标题" />
            </Form.Item>
          </div>

          {/* ── Row 2: TMDB ID + 年份 + 季号 ── */}
          <div className="grid grid-cols-3 gap-3">
            <Form.Item name="tmdbId" label="TMDB ID">
              <Input type="number" placeholder="可选" />
            </Form.Item>
            <Form.Item name="year" label="年份">
              <Input placeholder="2024" />
            </Form.Item>
            {mediaType === "tv" ? (
              <Form.Item name="season" label="季号">
                <Input type="number" placeholder="1" />
              </Form.Item>
            ) : (
              <div />
            )}
          </div>

          {/* ── Row 3 (TV only): 集数 ── */}
          {mediaType === "tv" && (
            <Form.Item name="episodes" label="指定集数">
              <Input placeholder="逗号分隔，如 1,2,3（留空 = 整季）" />
            </Form.Item>
          )}

          {/* ── Row 4: 分类 + 下载器 + 间隔 ── */}
          <div className="grid grid-cols-3 gap-3">
            <Form.Item name="category" label="分类">
              <Select
                options={categoryOptions}
                placeholder={t("category.placeholder")}
                allowClear
              />
            </Form.Item>
            <Form.Item name="downloadClientId" label="下载器">
              <Select options={clientOptions} placeholder="选择" allowClear />
            </Form.Item>
            <Form.Item
              name="intervalMinutes"
              label="检查间隔(分)"
              initialValue={5}
            >
              <Input type="number" />
            </Form.Item>
          </div>

          {/* ── Row 5: PT 站点 ── */}
          <Form.Item name="siteIds" label="指定 PT 站点" className="mb-2">
            <Select
              mode="multiple"
              options={siteOptions}
              placeholder="留空则搜索所有站点"
              allowClear
            />
          </Form.Item>

          {/* ── 海报 URL (可折叠) ── */}
          <Form.Item name="posterPath" label="海报 URL" className="mb-2">
            <Input placeholder="可选" />
          </Form.Item>

          {/* ── 过滤规则 ── */}
          <div className="border-t border-black/[0.06] dark:border-white/[0.08] pt-3 mt-1 mb-2">
            <h3 className="text-sm font-medium mb-2">过滤规则</h3>
          </div>

          <div className="grid grid-cols-3 gap-3">
            <Form.Item name="sources" label="来源">
              <Select
                mode="multiple"
                options={(filterOptions?.sources ?? []).map((s) => ({
                  label: s,
                  value: s,
                }))}
                placeholder="不限"
                allowClear
              />
            </Form.Item>
            <Form.Item name="resolutions" label="分辨率">
              <Select
                mode="multiple"
                options={(filterOptions?.resolutions ?? []).map((r) => ({
                  label: r,
                  value: r,
                }))}
                placeholder="不限"
                allowClear
              />
            </Form.Item>
            <Form.Item name="codecs" label="编码">
              <Select
                mode="multiple"
                options={(filterOptions?.codecs ?? []).map((c) => ({
                  label: c,
                  value: c,
                }))}
                placeholder="不限"
                allowClear
              />
            </Form.Item>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <Form.Item name="includeKeywords" label="包含关键词">
              <Input placeholder="可选" />
            </Form.Item>
            <Form.Item name="excludeKeywords" label="排除关键词">
              <Input placeholder="可选" />
            </Form.Item>
          </div>

          <div className="flex gap-6">
            <Form.Item name="freeOnly" valuePropName="checked">
              <Checkbox>仅免费种子</Checkbox>
            </Form.Item>
            <Form.Item name="excludeHr" valuePropName="checked">
              <Checkbox>排除半成品</Checkbox>
            </Form.Item>
          </div>

          {/* ── 高级筛选 ── */}
          <div className="border-t border-black/[0.06] dark:border-white/[0.08] pt-3 mt-1">
            <button
              type="button"
              className="flex items-center gap-1 text-sm font-medium text-muted-foreground hover:text-foreground cursor-pointer"
              onClick={() => setShowAdvanced(!showAdvanced)}
            >
              {showAdvanced ? (
                <ChevronDown size={14} />
              ) : (
                <ChevronRight size={14} />
              )}
              高级筛选
            </button>
          </div>

          {showAdvanced && (
            <div className="mt-2 space-y-2">
              <Form.Item name="releaseGroups" label="字幕组" className="mb-0">
                <Input placeholder="逗号分隔，如 CHD, FRDS" />
              </Form.Item>
              <div className="grid grid-cols-2 gap-3">
                <Form.Item name="minSize" label="最小 (GB)" className="mb-0">
                  <Input type="number" placeholder="0" />
                </Form.Item>
                <Form.Item name="maxSize" label="最大 (GB)" className="mb-0">
                  <Input type="number" placeholder="不限" />
                </Form.Item>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <Form.Item name="minSeeders" label="最少做种" className="mb-0">
                  <Input type="number" placeholder="0" />
                </Form.Item>
                <Form.Item name="maxSeeders" label="最多做种" className="mb-0">
                  <Input type="number" placeholder="不限" />
                </Form.Item>
              </div>
            </div>
          )}
        </Form>
      </ScrollArea>

      <div className="flex justify-end gap-2 px-6 py-3 border-t border-black/[0.06] dark:border-white/[0.08] shrink-0">
        <Button onClick={win.close}>取消</Button>
        <Button
          variant="primary"
          loading={submitting}
          onClick={() => void handleSubmit()}
        >
          确定
        </Button>
      </div>
    </div>
  );
}
