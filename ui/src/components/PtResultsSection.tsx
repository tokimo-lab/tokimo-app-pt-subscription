import {
  Button,
  Checkbox,
  Divider,
  Empty,
  FilterOutlined,
  Pagination,
  Popover,
  Select,
  Tag,
} from "@tokimo/ui";
import { useEffect, useMemo, useRef, useState } from "react";
import type { PtSearchResultWithSite, SiteSummary } from "../api/client";
import {
  extractFilterOptions,
  getDiscountLabel,
  getDownloadFactor,
  getUploadFactor,
  PT_PAGE_SIZE,
} from "./search-utils";
import TorrentCard from "./TorrentCard";

export interface SearchFilters {
  resolution: string[];
  videoCodec: string[];
  discount: string[];
  year: string[];
  season: string[];
  episode: string[];
  freeOnly: boolean;
}

const EMPTY_FILTERS: SearchFilters = {
  resolution: [],
  videoCodec: [],
  discount: [],
  year: [],
  season: [],
  episode: [],
  freeOnly: false,
};

interface PtResultsSectionProps {
  ptResults: PtSearchResultWithSite[];
  siteSummaries: SiteSummary[];
  activeSite: string;
  onSiteChange: (siteId: string) => void;
  onDownload: (torrent: PtSearchResultWithSite) => void;
}

export default function PtResultsSection({
  ptResults,
  siteSummaries,
  activeSite,
  onSiteChange,
  onDownload,
}: PtResultsSectionProps) {
  const sectionRef = useRef<HTMLDivElement>(null);
  const [filters, setFilters] = useState<SearchFilters>(EMPTY_FILTERS);
  const [ptPage, setPtPage] = useState({ current: 1, pageSize: PT_PAGE_SIZE });

  const dynamicOptions = useMemo(
    () => extractFilterOptions(ptResults),
    [ptResults],
  );

  const SEASON_EPISODE_RE =
    /S(\d{1,4})(?:\s*E(\d{1,4})(?:\s*-\s*E?(\d{1,4}))?)?/gi;

  // Filter by site tab first
  const siteFilteredResults = useMemo(() => {
    if (activeSite === "all") return ptResults;
    return ptResults.filter((r) => r.siteDbId === activeSite);
  }, [ptResults, activeSite]);

  // Apply filters
  const filteredPtResults = useMemo(() => {
    return siteFilteredResults.filter((torrent) => {
      if (filters.resolution.length > 0) {
        const normalized = (torrent.resolution ?? "").toLowerCase();
        const match = filters.resolution.some((v) => {
          const lower = v.toLowerCase();
          if (lower.includes("4k") || lower.includes("2160"))
            return normalized.includes("4k") || normalized.includes("2160");
          return normalized.includes(lower);
        });
        if (!match) return false;
      }

      if (filters.videoCodec.length > 0) {
        if (!filters.videoCodec.some((v) => torrent.videoCodec?.includes(v)))
          return false;
      }

      if (filters.discount.length > 0) {
        const dlFactor = getDownloadFactor(torrent);
        const ulFactor = getUploadFactor(torrent);
        const matchDiscount = filters.discount.some((d) => {
          if (d === "free") return dlFactor === 0;
          if (d === "half") return dlFactor === 0.5;
          if (d === "2x") return ulFactor >= 2;
          return false;
        });
        if (!matchDiscount) return false;
      }

      if (filters.year.length > 0) {
        const fullText = `${torrent.title} ${torrent.subtitle ?? ""}`;
        const yearRe = /(?:^|[\s.([])((?:19|20)\d{2})(?:[\s.)\]]|$)/g;
        const years: string[] = [];
        let m: RegExpExecArray | null;
        // biome-ignore lint/suspicious/noAssignInExpressions: regex exec loop
        while ((m = yearRe.exec(fullText)) !== null) years.push(m[1]);
        if (!filters.year.some((y) => years.includes(y))) return false;
      }

      if (filters.season.length > 0) {
        const fullText = `${torrent.title} ${torrent.subtitle ?? ""}`;
        const seasonNums: string[] = [];
        let m: RegExpExecArray | null;
        SEASON_EPISODE_RE.lastIndex = 0;
        // biome-ignore lint/suspicious/noAssignInExpressions: regex exec loop
        while ((m = SEASON_EPISODE_RE.exec(fullText)) !== null)
          seasonNums.push(m[1]);
        if (!filters.season.some((s) => seasonNums.includes(s))) return false;
      }

      if (filters.episode.length > 0) {
        const fullText = `${torrent.title} ${torrent.subtitle ?? ""}`;
        const epNums: string[] = [];
        let m: RegExpExecArray | null;
        SEASON_EPISODE_RE.lastIndex = 0;
        // biome-ignore lint/suspicious/noAssignInExpressions: regex exec loop
        while ((m = SEASON_EPISODE_RE.exec(fullText)) !== null) {
          if (m[2]) {
            const start = Number.parseInt(m[2], 10);
            const end = m[3] ? Number.parseInt(m[3], 10) : start;
            for (let e = start; e <= end; e++) epNums.push(String(e));
          }
        }
        if (!filters.episode.some((ep) => epNums.includes(ep))) return false;
      }

      if (filters.freeOnly && getDownloadFactor(torrent) !== 0) return false;
      return true;
    });
  }, [siteFilteredResults, filters]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: reset page on filter change
  useEffect(() => {
    setPtPage((prev) => ({ ...prev, current: 1 }));
  }, [filters, activeSite]);

  const paginatedPtResults = useMemo(() => {
    const start = (ptPage.current - 1) * ptPage.pageSize;
    return filteredPtResults.slice(start, start + ptPage.pageSize);
  }, [filteredPtResults, ptPage]);

  const handlePageChange = (page: number, pageSize: number) => {
    setPtPage({ current: page, pageSize });
    sectionRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
  };

  const hasActiveFilters =
    filters.resolution.length > 0 ||
    filters.videoCodec.length > 0 ||
    filters.discount.length > 0 ||
    filters.year.length > 0 ||
    filters.season.length > 0 ||
    filters.episode.length > 0 ||
    filters.freeOnly;

  const filterContent = (
    <div className="w-56 space-y-3">
      {dynamicOptions.resolutions.length > 0 && (
        <div>
          <div className="text-xs text-fg-muted mb-1">分辨率</div>
          <Select
            size="small"
            mode="multiple"
            value={filters.resolution}
            onChange={(v: string[]) =>
              setFilters((f) => ({ ...f, resolution: v }))
            }
            options={dynamicOptions.resolutions.map((r) => ({
              label: r,
              value: r,
            }))}
            placeholder="全部"
            className="w-full"
            allowClear
          />
        </div>
      )}

      {dynamicOptions.videoCodecs.length > 0 && (
        <div>
          <div className="text-xs text-fg-muted mb-1">视频编码</div>
          <Select
            size="small"
            mode="multiple"
            value={filters.videoCodec}
            onChange={(v: string[]) =>
              setFilters((f) => ({ ...f, videoCodec: v }))
            }
            options={dynamicOptions.videoCodecs.map((c) => ({
              label: c,
              value: c,
            }))}
            placeholder="全部"
            className="w-full"
            allowClear
          />
        </div>
      )}

      {dynamicOptions.discounts.length > 0 && (
        <div>
          <div className="text-xs text-fg-muted mb-1">促销</div>
          <Select
            size="small"
            mode="multiple"
            value={filters.discount}
            onChange={(v: string[]) =>
              setFilters((f) => ({ ...f, discount: v }))
            }
            options={dynamicOptions.discounts.map((d) => ({
              label: getDiscountLabel(d),
              value: d,
            }))}
            placeholder="全部"
            className="w-full"
            allowClear
          />
        </div>
      )}

      {dynamicOptions.years.length > 1 && (
        <div>
          <div className="text-xs text-fg-muted mb-1">年份</div>
          <Select
            size="small"
            mode="multiple"
            value={filters.year}
            onChange={(v: string[]) => setFilters((f) => ({ ...f, year: v }))}
            options={dynamicOptions.years.map((y) => ({
              label: y,
              value: y,
            }))}
            placeholder="全部"
            className="w-full"
            allowClear
          />
        </div>
      )}

      {dynamicOptions.seasons.length > 0 && (
        <div>
          <div className="text-xs text-fg-muted mb-1">季</div>
          <Select
            size="small"
            mode="multiple"
            value={filters.season}
            onChange={(v: string[]) => setFilters((f) => ({ ...f, season: v }))}
            options={dynamicOptions.seasons.map((s) => ({
              label: `S${s.padStart(2, "0")}`,
              value: s,
            }))}
            placeholder="全部"
            className="w-full"
            allowClear
          />
        </div>
      )}

      {dynamicOptions.episodes.length > 0 && (
        <div>
          <div className="text-xs text-fg-muted mb-1">集</div>
          <Select
            size="small"
            mode="multiple"
            value={filters.episode}
            onChange={(v: string[]) =>
              setFilters((f) => ({ ...f, episode: v }))
            }
            options={dynamicOptions.episodes.map((e) => ({
              label: `E${e.padStart(2, "0")}`,
              value: e,
            }))}
            placeholder="全部"
            className="w-full"
            allowClear
          />
        </div>
      )}

      <Divider className="!my-2" />
      <Checkbox
        checked={filters.freeOnly}
        onChange={(e) =>
          setFilters((f) => ({ ...f, freeOnly: e.target.checked }))
        }
      >
        仅免费
      </Checkbox>
    </div>
  );

  const activeFilterTags = useMemo(() => {
    const tags: { label: string; onClose: () => void }[] = [];
    if (filters.freeOnly) {
      tags.push({
        label: "免费",
        onClose: () => setFilters((f) => ({ ...f, freeOnly: false })),
      });
    }
    for (const r of filters.resolution) {
      tags.push({
        label: r,
        onClose: () =>
          setFilters((f) => ({
            ...f,
            resolution: f.resolution.filter((v) => v !== r),
          })),
      });
    }
    for (const c of filters.videoCodec) {
      tags.push({
        label: c,
        onClose: () =>
          setFilters((f) => ({
            ...f,
            videoCodec: f.videoCodec.filter((v) => v !== c),
          })),
      });
    }
    for (const d of filters.discount) {
      tags.push({
        label: getDiscountLabel(d),
        onClose: () =>
          setFilters((f) => ({
            ...f,
            discount: f.discount.filter((v) => v !== d),
          })),
      });
    }
    for (const y of filters.year) {
      tags.push({
        label: y,
        onClose: () =>
          setFilters((f) => ({ ...f, year: f.year.filter((v) => v !== y) })),
      });
    }
    for (const s of filters.season) {
      tags.push({
        label: `S${s.padStart(2, "0")}`,
        onClose: () =>
          setFilters((f) => ({
            ...f,
            season: f.season.filter((v) => v !== s),
          })),
      });
    }
    for (const e of filters.episode) {
      tags.push({
        label: `E${e.padStart(2, "0")}`,
        onClose: () =>
          setFilters((f) => ({
            ...f,
            episode: f.episode.filter((v) => v !== e),
          })),
      });
    }
    return tags;
  }, [filters]);

  const clearAllFilters = () =>
    setFilters({
      resolution: [],
      videoCodec: [],
      discount: [],
      year: [],
      season: [],
      episode: [],
      freeOnly: false,
    });

  return (
    <div ref={sectionRef} className="flex flex-col">
      {/* Site filter tabs */}
      {siteSummaries.length > 0 && (
        <div className="flex items-center gap-1 flex-wrap mb-3">
          <button
            type="button"
            onClick={() => onSiteChange("all")}
            className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors cursor-pointer ${
              activeSite === "all"
                ? "bg-[var(--color-accent)] text-white"
                : "bg-black/[0.04] dark:bg-white/[0.06] text-fg-muted hover:bg-black/[0.08] dark:hover:bg-white/[0.10]"
            }`}
          >
            全部 ({ptResults.length})
          </button>
          {siteSummaries
            .filter((s) => s.count > 0)
            .map((site) => (
              <button
                key={site.siteDbId}
                type="button"
                onClick={() => onSiteChange(site.siteDbId)}
                className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors cursor-pointer ${
                  activeSite === site.siteDbId
                    ? "bg-[var(--color-accent)] text-white"
                    : "bg-black/[0.04] dark:bg-white/[0.06] text-fg-muted hover:bg-black/[0.08] dark:hover:bg-white/[0.10]"
                }`}
              >
                {site.siteName} ({site.count})
              </button>
            ))}
        </div>
      )}

      {/* Filter bar */}
      <div className="flex flex-wrap items-center justify-between gap-2 mb-3">
        <div className="flex items-center gap-2">
          <Tag>
            {filteredPtResults.length}/{siteFilteredResults.length}
          </Tag>
        </div>
        <div className="flex items-center gap-2 flex-wrap">
          {activeFilterTags.map((tag) => (
            <Tag key={tag.label} closable onClose={tag.onClose}>
              {tag.label}
            </Tag>
          ))}
          {hasActiveFilters && (
            <Tag
              color="red"
              className="cursor-pointer"
              closable
              onClose={clearAllFilters}
            >
              清除筛选
            </Tag>
          )}
          <Popover
            content={filterContent}
            trigger="click"
            placement="bottomRight"
          >
            <Button icon={<FilterOutlined />} size="small">
              筛选
            </Button>
          </Popover>
        </div>
      </div>

      {/* Torrent list */}
      {filteredPtResults.length > 0 ? (
        <>
          <div className="divide-y divide-[var(--color-border-base)] rounded-xl overflow-hidden border border-border-base">
            {paginatedPtResults.map((torrent) => (
              <TorrentCard
                key={`${torrent.siteDbId}-${torrent.id}`}
                torrent={torrent}
                onDownload={onDownload}
              />
            ))}
          </div>

          {filteredPtResults.length > ptPage.pageSize && (
            <div className="flex justify-center mt-4 shrink-0">
              <Pagination
                current={ptPage.current}
                pageSize={ptPage.pageSize}
                total={filteredPtResults.length}
                onChange={handlePageChange}
                showSizeChanger
                pageSizeOptions={[10, 20, 40, 100]}
                showQuickJumper
                showTotal={(total) => `共 ${total} 条`}
              />
            </div>
          )}
        </>
      ) : (
        <Empty description="暂无搜索结果" />
      )}
    </div>
  );
}
