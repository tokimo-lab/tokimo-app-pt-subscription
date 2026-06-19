import { useWindowActions } from "@tokimo/sdk";
import { AutoComplete, Image, Popover, ScrollArea, Tag } from "@tokimo/ui";
import { ChevronDown, Clock, Film, Star, Trash2 } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { PtSearchResultWithSite, SiteSummary } from "../api/client";
import { categoriesApi, searchApi } from "../api/client";
import PtResultsSection from "./PtResultsSection";
import {
  addSearchHistory,
  clearSearchHistory,
  fetchSearchHistory,
  fetchTmdbSearch,
  removeSearchHistory,
  type SearchHistoryItem,
  type TmdbMedia,
} from "./search-api";
import { useDebounce } from "./search-hooks";
import { categoryLabel } from "./search-utils";

// ── Constants ────────────────────────────────────────────────────────────

const QUICK_TAGS = [
  "权力的游戏",
  "老友记",
  "黑镜",
  "三体",
  "流浪地球",
  "漫长的季节",
  "繁花",
  "庆余年",
];

const SKELETON_KEYS = ["sk-1", "sk-2", "sk-3", "sk-4", "sk-5", "sk-6"];

// ── Category Dropdown ────────────────────────────────────────────────────

interface CategoryOption {
  label: string;
  value: string;
}

function CategoryDropdown({
  options,
  selected,
  onChange,
}: {
  options: CategoryOption[];
  selected: string;
  onChange: (val: string) => void;
}) {
  const [open, setOpen] = useState(false);

  const displayText = useMemo(() => {
    if (!selected) return "全部分类";
    const opt = options.find((o) => o.value === selected);
    return opt?.label ?? "全部分类";
  }, [selected, options]);

  const selectItem = (id: string) => {
    onChange(id === selected ? "" : id);
    setOpen(false);
  };

  return (
    <Popover
      open={open}
      onOpenChange={setOpen}
      trigger="click"
      placement="bottomLeft"
      content={
        <div className="w-44 max-h-80 overflow-y-auto py-1">
          <button
            type="button"
            onClick={() => selectItem("")}
            className={`w-full text-left px-3 py-1.5 text-sm cursor-pointer hover:bg-black/[0.04] dark:hover:bg-white/[0.06] transition-colors ${
              !selected ? "text-[var(--color-accent)] font-medium" : ""
            }`}
          >
            全部分类
          </button>
          {options.map((item) => (
            <button
              key={item.value}
              type="button"
              onClick={() => selectItem(item.value)}
              className={`w-full text-left px-3 py-1.5 text-sm cursor-pointer hover:bg-black/[0.04] dark:hover:bg-white/[0.06] transition-colors ${
                selected === item.value
                  ? "text-[var(--color-accent)] font-medium"
                  : "text-[var(--color-fg-primary)]"
              }`}
            >
              {item.label}
            </button>
          ))}
        </div>
      }
    >
      <button
        type="button"
        className="flex items-center gap-1.5 px-3 text-sm text-fg-muted hover:text-fg-base cursor-pointer select-none shrink-0 max-w-[140px] self-stretch"
      >
        <span className="truncate">{displayText}</span>
        <ChevronDown
          size={14}
          className={`transition-transform ${open ? "rotate-180" : ""}`}
        />
      </button>
    </Popover>
  );
}

// ── Suggestion option renderer ───────────────────────────────────────────

function SuggestionLabel({ media }: { media: TmdbMedia }) {
  const year = media.releaseDate?.slice(0, 4);
  const isTv = media.mediaType === "tv";
  const rating = media.voteAverage?.toFixed(1);

  return (
    <div className="flex items-center gap-3 py-1">
      <div className="w-8 h-11 rounded overflow-hidden bg-[var(--color-fill-skeleton)] flex-shrink-0">
        {media.posterPath ? (
          <Image
            src={`https://image.tmdb.org/t/p/w92${media.posterPath}`}
            alt={media.title}
            className="!w-full !h-full object-cover"
            rootClassName="!w-full !h-full"
            preview={false}
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center">
            <Film size={12} className="text-fg-muted" />
          </div>
        )}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5">
          <span className="text-sm font-medium truncate">{media.title}</span>
          <span
            className={`px-1 py-0.5 rounded text-[9px] font-semibold text-white leading-tight flex-shrink-0 ${
              isTv ? "bg-green-600/80" : "bg-blue-600/80"
            }`}
          >
            {isTv ? "剧集" : "电影"}
          </span>
        </div>
        <div className="flex items-center gap-1.5 mt-0.5 text-xs text-fg-muted">
          {year && <span>{year}</span>}
          {rating && (
            <span className="flex items-center gap-0.5">
              <Star size={9} />
              {rating}
            </span>
          )}
          {media.originalTitle && media.originalTitle !== media.title && (
            <span className="truncate">{media.originalTitle}</span>
          )}
        </div>
      </div>
    </div>
  );
}

// ── Main Page ────────────────────────────────────────────────────────────

export default function SearchPage() {
  const windowActions = useWindowActions();
  const { t } = useTranslation();
  const [keyword, setKeyword] = useState("");
  const [suggestions, setSuggestions] = useState<TmdbMedia[]>([]);
  const [history, setHistory] = useState<SearchHistoryItem[]>([]);
  const [tmdbResults, setTmdbResults] = useState<TmdbMedia[]>([]);
  const [ptResults, setPtResults] = useState<PtSearchResultWithSite[]>([]);
  const [siteSummaries, setSiteSummaries] = useState<SiteSummary[]>([]);
  const [activeSite, setActiveSite] = useState<string>("all");
  const [hasSearched, setHasSearched] = useState(false);
  const [tmdbLoading, setTmdbLoading] = useState(false);
  const [selectedCategory, setSelectedCategory] = useState("");
  const [lastSearchKeyword, setLastSearchKeyword] = useState("");
  const [categorySlugs, setCategorySlugs] = useState<string[]>([]);

  const categoryOptions = useMemo<CategoryOption[]>(
    () =>
      categorySlugs.map((slug) => ({
        label: categoryLabel(slug, t),
        value: slug,
      })),
    [categorySlugs, t],
  );

  const suggestionCache = useRef<Map<string, TmdbMedia[]>>(new Map());
  const debouncedKeyword = useDebounce(keyword, 300);

  const autocompleteOptions = useMemo(
    () =>
      suggestions.map((m) => ({
        value: m.title,
        label: <SuggestionLabel media={m} />,
      })),
    [suggestions],
  );

  useEffect(() => {
    fetchSearchHistory()
      .then(setHistory)
      .catch(() => {});
    categoriesApi
      .list()
      .then((data) => {
        setCategorySlugs(data.categories.map((c) => c.slug));
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    const trimmed = debouncedKeyword.trim();
    if (trimmed.length < 2 || hasSearched) {
      setSuggestions([]);
      return;
    }

    const cached = suggestionCache.current.get(trimmed);
    if (cached) {
      setSuggestions(cached);
      return;
    }

    let cancelled = false;
    fetchTmdbSearch(trimmed)
      .then((data) => {
        if (cancelled) return;
        const results = data.tmdbResults ?? [];
        suggestionCache.current.set(trimmed, results);
        setSuggestions(results);
      })
      .catch(() => {});

    return () => {
      cancelled = true;
    };
  }, [debouncedKeyword, hasSearched]);

  const executeSearch = useCallback(
    (kw: string, cat?: string) => {
      const trimmed = kw.trim();
      if (!trimmed) return;

      const category = cat ?? selectedCategory;
      const categories = category ? [category] : [];
      setKeyword(trimmed);
      setHasSearched(true);
      setTmdbLoading(true);
      setSuggestions([]);
      setLastSearchKeyword(trimmed);

      fetchTmdbSearch(trimmed)
        .then((data) => setTmdbResults(data.tmdbResults ?? []))
        .catch(() => {})
        .finally(() => setTmdbLoading(false));

      searchApi
        .searchPt(trimmed, [], categories)
        .then((data) => {
          setPtResults(data.results);
          setSiteSummaries(data.siteSummaries);
          setActiveSite("all");
        })
        .catch(() => {});

      addSearchHistory(trimmed)
        .then(setHistory)
        .catch(() => {});
    },
    [selectedCategory],
  );

  const handleSearch = (value: string) => {
    if (hasSearched) {
      setHasSearched(false);
      setTmdbResults([]);
      setPtResults([]);
      setSiteSummaries([]);
    }
    setKeyword(value);
  };

  const handleCategoryChange = useCallback(
    (cat: string) => {
      setSelectedCategory(cat);
      if (hasSearched && lastSearchKeyword) {
        const categories = cat ? [cat] : [];
        searchApi
          .searchPt(lastSearchKeyword, [], categories)
          .then((data) => {
            setPtResults(data.results);
            setSiteSummaries(data.siteSummaries);
            setActiveSite("all");
          })
          .catch(() => {});
      }
    },
    [hasSearched, lastSearchKeyword],
  );

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.nativeEvent.isComposing) {
      executeSearch(keyword);
    }
  };

  const handleSelect = (_value: string, option: { value: string }) => {
    const media = suggestions.find((m) => m.title === option.value);
    if (media) {
      executeSearch(media.title);
    } else {
      executeSearch(option.value);
    }
  };

  const handleHistoryClick = (kw: string) => {
    executeSearch(kw);
  };

  const handleHistoryRemove = async (kw: string) => {
    try {
      const updated = await removeSearchHistory(kw);
      setHistory(updated);
    } catch {}
  };

  const handleClearHistory = async () => {
    try {
      await clearSearchHistory();
      setHistory([]);
    } catch {}
  };

  const handleQuickTag = (tag: string) => {
    executeSearch(tag);
  };

  const _handlePtSearchFromTmdb = (tmdbTitle: string) => {
    setKeyword(tmdbTitle);
    searchApi
      .searchPt(tmdbTitle, [], selectedCategory ? [selectedCategory] : [])
      .then((data) => {
        setPtResults(data.results);
        setSiteSummaries(data.siteSummaries);
        setActiveSite("all");
      })
      .catch(() => {});
    addSearchHistory(tmdbTitle)
      .then(setHistory)
      .catch(() => {});
  };

  const handleSubscribe = (media: TmdbMedia) => {
    windowActions.openModalWindow({
      component: () => import("./SubscriptionFormWindow"),
      title: "新建订阅",
      width: 680,
      height: 700,
      metadata: {
        editingSubscription: null,
        prefilled: {
          mediaType: media.mediaType === "tv" ? "tv" : "movie",
          tmdbId: media.id,
          title: media.title,
          year: media.releaseDate?.slice(0, 4),
          posterPath: media.posterPath,
        },
        onSaved: () => {
          // no-op, subscriptions page will refresh on next visit
        },
      },
    });
  };

  const handleDownload = (torrent: PtSearchResultWithSite) => {
    windowActions.openModalWindow({
      component: () => import("./DownloadConfirmWindow"),
      title: "下载确认",
      width: 600,
      height: 700,
      metadata: {
        torrent,
        onSuccess: () => {
          // no-op
        },
      },
    });
  };

  const showInitialState = !hasSearched;

  return (
    <div className="flex flex-col h-full">
      {/* Search Input */}
      <div
        className={`transition-all duration-300 ${showInitialState ? "flex-1 flex flex-col items-center justify-center" : ""}`}
      >
        <div className={`w-full ${showInitialState ? "max-w-2xl px-4" : ""}`}>
          {showInitialState && (
            <div className="text-center mb-8">
              <h1 className="text-2xl font-bold mb-2">剧集搜索</h1>
              <p className="text-fg-muted text-sm">
                搜索影视资源，自动匹配 TMDB 数据并在 PT 站点查找种子
              </p>
            </div>
          )}

          {/* Unified Search Bar */}
          <div
            className={`flex items-center rounded-xl border border-border-base bg-black/[0.03] dark:bg-white/[0.05] transition-all ${
              showInitialState
                ? "shadow-lg focus-within:border-[var(--color-accent)] focus-within:ring-2 focus-within:ring-[var(--color-accent)]/20"
                : "focus-within:border-[var(--color-accent)]"
            }`}
          >
            {!showInitialState && (
              <span className="text-sm font-semibold text-fg-muted px-3 shrink-0">
                剧集搜索
              </span>
            )}

            {categoryOptions.length > 0 && (
              <>
                {!showInitialState && (
                  <div className="w-px h-5 bg-border-base shrink-0" />
                )}
                <CategoryDropdown
                  options={categoryOptions}
                  selected={selectedCategory}
                  onChange={handleCategoryChange}
                />
                <div className="w-px h-5 bg-border-base shrink-0" />
              </>
            )}

            <AutoComplete
              value={keyword}
              onChange={handleSearch}
              onSelect={handleSelect}
              onKeyDown={handleKeyDown}
              options={autocompleteOptions}
              filterOption={false}
              placeholder="输入剧名或电影名搜索..."
              allowClear
              size="large"
              className="flex-1 [&>div]:!border-0 [&>div]:!rounded-none [&>div]:!ring-0 [&>div]:!bg-transparent"
            />
          </div>

          {/* Initial State: Quick Tags + History */}
          {showInitialState && (
            <div className="mt-8 space-y-6">
              <div className="text-center">
                <div className="text-xs text-fg-muted mb-3">热门搜索</div>
                <div className="flex flex-wrap justify-center gap-2">
                  {QUICK_TAGS.map((tag) => (
                    <button
                      key={tag}
                      type="button"
                      onClick={() => handleQuickTag(tag)}
                      className="px-3 py-1.5 rounded-full text-sm bg-black/[0.04] dark:bg-white/[0.06] text-fg-muted hover:bg-black/[0.08] dark:hover:bg-white/[0.10] hover:text-fg-base transition-colors cursor-pointer"
                    >
                      {tag}
                    </button>
                  ))}
                </div>
              </div>

              {history.length > 0 && (
                <div className="max-w-md mx-auto">
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-1.5 text-xs text-fg-muted">
                      <Clock size={12} />
                      最近搜索
                    </div>
                    <button
                      type="button"
                      onClick={handleClearHistory}
                      className="text-xs text-fg-muted hover:text-fg-base cursor-pointer"
                    >
                      清除
                    </button>
                  </div>
                  <div className="space-y-1">
                    {history.slice(0, 10).map((item) => (
                      <div
                        key={item.id}
                        className="flex items-center gap-2 group"
                      >
                        <button
                          type="button"
                          onClick={() => handleHistoryClick(item.keyword)}
                          className="flex-1 text-left px-3 py-2 rounded-lg text-sm hover:bg-black/[0.04] dark:hover:bg-white/[0.06] transition-colors cursor-pointer truncate"
                        >
                          {item.keyword}
                        </button>
                        <button
                          type="button"
                          onClick={() => handleHistoryRemove(item.keyword)}
                          className="p-1.5 rounded text-fg-muted hover:text-fg-base opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer"
                        >
                          <Trash2 size={12} />
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Results (after search) */}
      {hasSearched && (
        <div className="flex-1 min-h-0 overflow-auto space-y-4 pt-2">
          {tmdbLoading ? (
            <div className="rounded-xl border border-border-base bg-black/[0.02] dark:bg-white/[0.03] p-4">
              <div className="flex items-center gap-2 mb-3">
                <Film size={16} className="text-fg-muted" />
                <h5 className="text-sm font-semibold">TMDB</h5>
              </div>
              <div className="flex gap-3 overflow-hidden">
                {SKELETON_KEYS.map((k) => (
                  <div
                    key={k}
                    className="shrink-0 w-[130px] h-[240px] rounded-lg bg-[var(--color-fill-skeleton)] animate-pulse"
                  />
                ))}
              </div>
            </div>
          ) : tmdbResults.length > 0 ? (
            <div className="rounded-xl border border-border-base bg-black/[0.02] dark:bg-white/[0.03] p-4 shrink-0">
              <div className="flex items-center gap-2 mb-3">
                <Film size={16} className="text-fg-muted" />
                <h5 className="text-sm font-semibold">TMDB</h5>
                <Tag>{tmdbResults.length}</Tag>
              </div>
              <ScrollArea direction="horizontal" className="pb-2">
                <div className="flex gap-3">
                  {tmdbResults.map((media) => (
                    <TmdbResultCard
                      key={media.id}
                      media={media}
                      onSubscribe={handleSubscribe}
                    />
                  ))}
                </div>
              </ScrollArea>
            </div>
          ) : null}

          {(ptResults.length > 0 || siteSummaries.length > 0) && (
            <PtResultsSection
              ptResults={ptResults}
              siteSummaries={siteSummaries}
              activeSite={activeSite}
              onSiteChange={setActiveSite}
              onDownload={handleDownload}
            />
          )}
        </div>
      )}
    </div>
  );
}

// ── TMDB Card (for results) ──────────────────────────────────────────────

function TmdbResultCard({
  media,
  onSubscribe,
}: {
  media: TmdbMedia;
  onSubscribe: (media: TmdbMedia) => void;
}) {
  const year = media.releaseDate?.slice(0, 4);
  const rating = media.voteAverage?.toFixed(1);
  const isTv = media.mediaType === "tv";
  const detailUrl = `https://www.themoviedb.org/${media.mediaType}/${media.id}`;

  return (
    <div className="group shrink-0 w-[130px] rounded-lg overflow-hidden bg-black/40 shadow-md hover:shadow-xl transition-shadow duration-200 flex flex-col">
      <a
        href={detailUrl}
        target="_blank"
        rel="noopener noreferrer"
        className="block"
      >
        <div className="relative aspect-[2/3] bg-[var(--color-fill-skeleton)] overflow-hidden">
          {media.posterPath ? (
            <Image
              src={`https://image.tmdb.org/t/p/w300${media.posterPath}`}
              alt={media.title}
              className="!w-full !h-full object-cover"
              rootClassName="!w-full !h-full"
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center p-3 text-center text-fg-muted text-sm">
              {media.title}
            </div>
          )}

          <span
            className={`absolute top-2 left-2 px-1.5 py-0.5 rounded text-[11px] font-semibold text-white leading-tight ${
              isTv ? "bg-green-600/80" : "bg-blue-600/80"
            }`}
          >
            {isTv ? "剧集" : "电影"}
          </span>

          <div
            className="absolute inset-x-0 bottom-0 pointer-events-none"
            style={{
              height: "40%",
              background:
                "linear-gradient(to top, rgba(0,0,0,0.85) 0%, rgba(0,0,0,0) 100%)",
            }}
          />

          {year && (
            <span className="absolute bottom-1.5 left-2.5 text-white/80 font-bold text-base drop-shadow-md">
              {year}
            </span>
          )}
          {rating && (
            <span className="absolute bottom-1.5 right-2.5 text-white/80 font-bold text-sm drop-shadow-md">
              {rating}
            </span>
          )}
        </div>
      </a>

      <div className="px-2 py-2 min-w-0 text-center">
        <a
          href={detailUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="block truncate text-xs font-semibold !text-neutral-100/80 hover:!text-neutral-100 transition-colors no-underline"
          title={media.title}
        >
          {media.title}
        </a>
        {media.imdbRating && (
          <div className="text-[11px] text-fg-muted mt-0.5">
            IMDb {media.imdbRating.toFixed(1)}
          </div>
        )}
      </div>

      <div className="flex px-2 pb-2 mt-auto">
        <button
          type="button"
          onClick={() => onSubscribe(media)}
          className="flex-1 px-2 py-1.5 rounded text-xs font-medium bg-[var(--color-accent)] text-white hover:opacity-90 transition-opacity cursor-pointer"
        >
          订阅
        </button>
      </div>
    </div>
  );
}
