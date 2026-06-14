/**
 * 订阅管理服务
 * CRUD + 订阅执行逻辑 (搜索 → 过滤 → 匹配 → 下载)
 * 支持电影订阅和 TV 剧集订阅（按季，支持集数级追踪）
 */

import { randomUUID } from "node:crypto";
import type { Subscription as PrismaSubscription } from "@prisma/client";
import type {
  CreateSubscriptionInput,
  PtTorrent,
  Subscription,
  SubscriptionDebugInfo,
  SubscriptionLogEntry,
  SubscriptionStatus,
  UpdateSubscriptionInput,
  UserRole,
} from "@tokiomo/types";
import { TRPCError } from "@trpc/server";
import { prisma } from "../../db/client";
import { extractOriginalUrl, toProxyUrl } from "../../lib/image-proxy";
import { parseMediaFilename } from "../../lib/media-parser";
import { OmdbClient } from "../../lib/omdb-client";
import { TmdbClient } from "../../lib/tmdb-client";
import { selectTorrentFilesByEpisodes } from "../../lib/torrent-file-filter";
import { assertOwnership, shouldShowAll } from "../../trpc/auth-helpers";
import { downloadManageService } from "../download-manage/download-manage.service";
import { notificationService } from "../notification/notification.service";
import { searchService } from "../search/search.service";
import { subscriptionFilterService } from "../subscription-filter/subscription-filter.service";
import { executeAdultSeriesSubscription } from "./adult-series.service";
import {
  createRunContext,
  deleteSubscriptionLogs,
  getRunSummaries,
  readRecentLogs,
  trimLogs,
} from "./subscription.logger";
import {
  buildEpisodeMapping,
  buildSearchKeywords,
  checkTvCompletion,
  type EpisodeMapping,
  matchTorrents,
  matchTorrentsForTv,
  updateNextCheck,
} from "./subscription-matcher";

// 转换数据库实体为 API 输出
export const toSubscriptionOutput = (
  row: PrismaSubscription & {
    filterName?: string | null;
    filterNames?: string[] | null;
    createdByName?: string | null;
  },
): Subscription => {
  // 优先使用 filterIds，向后兼容 filterId
  const filterIds = (row.filterIds as string[] | null)?.length
    ? (row.filterIds as string[])
    : row.filterId
      ? [row.filterId]
      : null;
  const filterNames = row.filterNames?.length
    ? row.filterNames
    : row.filterName
      ? [row.filterName]
      : null;

  return {
    id: row.id,
    subscriptionMode: (row.subscriptionMode ?? "tmdb") as
      | "tmdb"
      | "adult_series",
    mediaType: row.mediaType as "movie" | "tv" | "adult",
    tmdbId: row.tmdbId ? Number(row.tmdbId) : null,
    title: row.title,
    year: row.year ?? null,
    posterPath: row.posterPath
      ? toProxyUrl(extractOriginalUrl(row.posterPath))
      : null,
    season: row.season != null ? Number(row.season) : null,
    episodes: (row.episodes as number[] | null) ?? null,
    seriesPrefix: row.seriesPrefix ?? null,
    metadataSource: row.metadataSource ?? null,
    maxDownloadsPerRun: row.maxDownloadsPerRun ?? 10,
    filterIds: filterIds ?? null,
    filterNames: filterNames ?? null,
    filterOverrides: row.filterOverrides ?? null,
    status: row.status as SubscriptionStatus,
    intervalMinutes: Number(row.intervalMinutes ?? 30),
    siteIds: (row.siteIds as string[] | null) ?? null,
    downloadClientId: row.downloadClientId ?? null,
    targetLibraryId: row.targetLibraryId ?? null,
    lastCheckedAt: row.lastCheckedAt?.toISOString() ?? null,
    nextCheckAt: row.nextCheckAt?.toISOString() ?? null,
    createdBy: row.createdBy ?? null,
    createdByName: row.createdByName ?? undefined,
    createdAt: row.createdAt!.toISOString(),
    updatedAt: row.updatedAt!.toISOString(),
  };
};

export class SubscriptionService {
  /** 正在执行的订阅: subscriptionId → runId */
  private activeRuns = new Map<string, string>();

  // ==================== CRUD ====================

  async list(
    userId: string,
    userRole: UserRole | undefined,
  ): Promise<Subscription[]> {
    const showAll = await shouldShowAll(userId, userRole);
    const rows = await prisma.subscription.findMany({
      where: showAll ? undefined : { createdBy: userId },
      include: { filter: true, creator: true },
      orderBy: { createdAt: "desc" },
    });

    // Resolve all filterIds → filterNames
    const allFilterIds = new Set<string>();
    for (const r of rows) {
      const ids = (r.filterIds as string[] | null) ?? [];
      for (const id of ids) allFilterIds.add(id);
      if (r.filterId) allFilterIds.add(r.filterId);
    }

    let filterNameMap = new Map<string, string>();
    if (allFilterIds.size > 0) {
      const allFilters = await prisma.subscriptionFilter.findMany({
        select: { id: true, name: true },
      });
      filterNameMap = new Map(allFilters.map((f) => [f.id, f.name]));
    }

    return rows.map((r) => {
      const ids = (r.filterIds as string[] | null)?.length
        ? (r.filterIds as string[])
        : r.filterId
          ? [r.filterId]
          : [];
      const filterNames = ids
        .map((id) => filterNameMap.get(id) ?? id)
        .filter(Boolean);

      return toSubscriptionOutput({
        ...r,
        filterName: r.filter?.name ?? null,
        filterNames: filterNames.length ? filterNames : null,
        createdByName: r.creator?.name ?? null,
      });
    });
  }

  async getById(id: string): Promise<Subscription | null> {
    const row = await prisma.subscription.findUnique({
      where: { id },
      include: { filter: true, creator: true },
    });

    if (!row) return null;

    // Resolve filterIds → filterNames
    const filterIdsList = (row.filterIds as string[] | null)?.length
      ? (row.filterIds as string[])
      : row.filterId
        ? [row.filterId]
        : [];
    let filterNames: string[] = [];
    if (filterIdsList.length > 0) {
      const allFilters = await prisma.subscriptionFilter.findMany({
        select: { id: true, name: true },
      });
      const nameMap = new Map(allFilters.map((f) => [f.id, f.name]));
      filterNames = filterIdsList
        .map((fid) => nameMap.get(fid) ?? fid)
        .filter(Boolean);
    }

    return toSubscriptionOutput({
      ...row,
      filterName: row.filter?.name ?? null,
      filterNames: filterNames.length ? filterNames : null,
      createdByName: row.creator?.name ?? null,
    });
  }

  async create(
    input: CreateSubscriptionInput,
    userId?: string,
  ): Promise<Subscription> {
    const now = new Date();

    const created = await prisma.subscription.create({
      data: {
        subscriptionMode: input.subscriptionMode ?? "tmdb",
        mediaType: input.mediaType,
        tmdbId: input.tmdbId != null ? String(input.tmdbId) : null,
        title: input.title,
        year: input.year ?? null,
        posterPath: input.posterPath ?? null,
        season: input.season != null ? String(input.season) : null,
        episodes: input.episodes ?? undefined,
        seriesPrefix: input.seriesPrefix ?? null,
        metadataSource: input.metadataSource ?? null,
        maxDownloadsPerRun: input.maxDownloadsPerRun ?? 10,
        filterId: input.filterIds?.[0] ?? null,
        filterIds: input.filterIds ?? undefined,
        filterOverrides: input.filterOverrides ?? undefined,
        status: "active",
        intervalMinutes: String(input.intervalMinutes ?? 30),
        siteIds: input.siteIds ?? undefined,
        downloadClientId: input.downloadClientId ?? null,
        targetLibraryId: input.targetLibraryId ?? null,
        lastCheckedAt: null,
        nextCheckAt: now,
        createdBy: userId ?? null,
      },
    });

    // 创建后立即后台执行
    this.executeInBackground(created.id);

    return toSubscriptionOutput(created);
  }

  async update(input: UpdateSubscriptionInput): Promise<Subscription> {
    const sets: Record<string, unknown> = { updatedAt: new Date() };
    if (input.season !== undefined)
      sets.season = input.season != null ? String(input.season) : null;
    if (input.episodes !== undefined) sets.episodes = input.episodes;
    if (input.filterIds !== undefined) {
      sets.filterIds = input.filterIds;
      sets.filterId = input.filterIds?.[0] ?? null; // backward compat
    }
    if (input.filterOverrides !== undefined)
      sets.filterOverrides = input.filterOverrides;
    if (input.status !== undefined) sets.status = input.status;
    if (input.intervalMinutes !== undefined)
      sets.intervalMinutes = String(input.intervalMinutes);
    if (input.siteIds !== undefined) sets.siteIds = input.siteIds;
    if (input.downloadClientId !== undefined)
      sets.downloadClientId = input.downloadClientId;
    if (input.targetLibraryId !== undefined)
      sets.targetLibraryId = input.targetLibraryId;
    if (input.maxDownloadsPerRun !== undefined)
      sets.maxDownloadsPerRun = input.maxDownloadsPerRun;

    const existing = await prisma.subscription.findUnique({
      where: { id: input.id },
    });
    if (!existing) {
      throw new TRPCError({ code: "NOT_FOUND", message: "订阅不存在" });
    }
    const updated = await prisma.subscription.update({
      where: { id: input.id },
      data: sets,
    });
    return toSubscriptionOutput(updated);
  }

  async delete(id: string): Promise<void> {
    const existing = await this.getById(id);
    if (!existing) {
      throw new TRPCError({ code: "NOT_FOUND", message: "订阅不存在" });
    }
    await prisma.subscription.delete({ where: { id } });
    // 清理日志文件
    await deleteSubscriptionLogs(id);
  }

  /** 校验订阅所有权，不满足时抛出 FORBIDDEN */
  async assertOwner(
    id: string,
    userId: string,
    userRole: UserRole | undefined,
  ): Promise<void> {
    const sub = await prisma.subscription.findUnique({
      where: { id },
      select: { createdBy: true },
    });
    if (!sub) {
      throw new TRPCError({ code: "NOT_FOUND", message: "订阅不存在" });
    }
    assertOwnership(sub.createdBy, userId, userRole);
  }

  // ==================== 异步执行 ====================

  /**
   * 后台异步执行订阅，立即返回 runId
   * 前端可用 runId 轮询日志
   */
  executeInBackground(id: string): string {
    const runId = randomUUID();
    this.activeRuns.set(id, runId);
    // fire-and-forget
    this.executeSubscription(id, runId)
      .catch(() => {})
      .finally(() => {
        // 执行完成后移除
        if (this.activeRuns.get(id) === runId) {
          this.activeRuns.delete(id);
        }
      });
    return runId;
  }

  /**
   * 获取订阅当前活跃执行的 runId（无活跃返回 null）
   */
  getActiveRunId(subscriptionId: string): string | null {
    return this.activeRuns.get(subscriptionId) ?? null;
  }

  /**
   * 获取某次执行的日志
   */
  async getRunLogs(
    subscriptionId: string,
    runId: string,
  ): Promise<SubscriptionLogEntry[]> {
    const all = await readRecentLogs(subscriptionId, 10000);
    return all.filter((e) => e.runId === runId);
  }

  // ==================== 调试信息 ====================

  async getDebugInfo(id: string): Promise<SubscriptionDebugInfo> {
    const sub = await this.getById(id);
    if (!sub) {
      throw new TRPCError({ code: "NOT_FOUND", message: "订阅不存在" });
    }

    const activeRunId = this.getActiveRunId(id);
    const recentRuns = await getRunSummaries(id, 20);

    // 标记正在执行的 run
    const runsWithStatus = recentRuns.map((r) => ({
      ...r,
      isRunning: activeRunId !== null && r.runId === activeRunId,
    }));

    const totalRuns = recentRuns.length;
    const successfulDownloads = recentRuns.filter((r) => r.downloaded).length;
    const lastSuccess = recentRuns.find((r) => r.downloaded);

    return {
      subscription: sub,
      totalRuns,
      successfulDownloads,
      lastMatchedAt: lastSuccess?.startedAt ?? null,
      activeRunId,
      recentRuns: runsWithStatus,
    };
  }

  async getRecentLogs(id: string, limit = 200) {
    return readRecentLogs(id, limit);
  }

  // ==================== TMDB 辅助 ====================

  /**
   * 创建 TMDB 客户端（如已配置）
   */
  private async createTmdbClient(): Promise<TmdbClient | null> {
    const settings = await prisma.tmdbSettings.findFirst();
    const apiKey = settings?.apiKey || process.env.TMDB_API_KEY;
    if (!apiKey) return null;
    return new TmdbClient({ apiKey, language: "zh-CN" });
  }

  /**
   * 创建 OMDb 客户端（如已配置）
   */
  private async createOmdbClient(): Promise<OmdbClient | null> {
    const settings = await prisma.omdbSettings.findFirst();
    const apiKey = settings?.apiKey || process.env.OMDB_API_KEY;
    if (!apiKey) return null;
    return new OmdbClient({ apiKey });
  }

  /**
   * 从 TMDB 获取某季的有效集数列表，TMDB 无数据时 fallback 到 OMDb
   */
  private async fetchValidEpisodes(
    tmdbId: number,
    season: number,
    logFn?: (message: string) => Promise<void>,
  ): Promise<number[] | null> {
    // 1. 尝试 TMDB
    const tmdbResult = await this.fetchValidEpisodesFromTmdb(
      tmdbId,
      season,
      logFn,
    );
    if (tmdbResult) return tmdbResult;

    // 2. Fallback: 通过 TMDB 获取 IMDb ID，然后用 OMDb 获取季度集数
    return this.fetchValidEpisodesFromOmdb(tmdbId, season, logFn);
  }

  private async fetchValidEpisodesFromTmdb(
    tmdbId: number,
    season: number,
    logFn?: (message: string) => Promise<void>,
  ): Promise<number[] | null> {
    try {
      const client = await this.createTmdbClient();
      if (!client) {
        await logFn?.("TMDB 未配置 API Key");
        return null;
      }
      const detail = await client.getTvSeasonDetail(tmdbId, season);
      if (!detail.episodes || detail.episodes.length === 0) {
        await logFn?.(
          `TMDB 返回空集数列表 (tmdbId=${tmdbId}, season=${season})，尝试 OMDb`,
        );
        return null;
      }
      return detail.episodes.map((e) => e.episode_number);
    } catch (err) {
      await logFn?.(
        `TMDB 获取季度详情失败: ${err instanceof Error ? err.message : "未知错误"}，尝试 OMDb`,
      );
      return null;
    }
  }

  private async fetchValidEpisodesFromOmdb(
    tmdbId: number,
    season: number,
    logFn?: (message: string) => Promise<void>,
  ): Promise<number[] | null> {
    try {
      // 先从 TMDB 获取 IMDb ID
      const tmdbClient = await this.createTmdbClient();
      if (!tmdbClient) return null;

      const tvDetail = await tmdbClient.getTvDetail(tmdbId);
      const imdbId = tvDetail.imdbId;
      if (!imdbId) {
        await logFn?.("无法从 TMDB 获取 IMDb ID，跳过 OMDb 集数验证");
        return null;
      }

      const omdbClient = await this.createOmdbClient();
      if (!omdbClient) {
        await logFn?.("OMDb 未配置 API Key，跳过集数验证");
        return null;
      }

      const seasonDetail = await omdbClient.getSeasonDetail(imdbId, season);
      if (!seasonDetail?.Episodes || seasonDetail.Episodes.length === 0) {
        await logFn?.(
          `OMDb 返回空集数列表 (imdbId=${imdbId}, season=${season})`,
        );
        return null;
      }

      const episodes = seasonDetail.Episodes.map((e) =>
        Number.parseInt(e.Episode, 10),
      ).filter((n) => !Number.isNaN(n));

      await logFn?.(`通过 OMDb (IMDb: ${imdbId}) 获取到有效集数`);
      return episodes.length > 0 ? episodes : null;
    } catch (err) {
      await logFn?.(
        `OMDb 获取季度详情失败: ${err instanceof Error ? err.message : "未知错误"}`,
      );
      return null;
    }
  }

  /**
   * 计算当前季的集数偏移量（offset）
   * 遍历所有前季，累加各季集数，用于绝对集数 → 相对集数的转换
   * @returns offset 数字，或 null 表示无法计算
   */
  private async calculateSeasonOffset(
    tmdbId: number,
    currentSeason: number,
    logFn?: (message: string) => Promise<void>,
  ): Promise<number | null> {
    if (currentSeason <= 1) return 0;

    let totalOffset = 0;
    const seasonCounts: string[] = [];

    for (let s = 1; s < currentSeason; s++) {
      const episodes = await this.fetchValidEpisodes(tmdbId, s);
      if (!episodes || episodes.length === 0) {
        await logFn?.(
          `无法获取 S${String(s).padStart(2, "0")} 集数信息，跳过绝对集数映射`,
        );
        return null;
      }
      totalOffset += episodes.length;
      seasonCounts.push(`S${String(s).padStart(2, "0")}=${episodes.length}`);
    }

    await logFn?.(
      `前 ${currentSeason - 1} 季累计集数: ${totalOffset} (${seasonCounts.join(", ")})`,
    );
    return totalOffset;
  }

  // ==================== 执行逻辑 ====================

  /**
   * 查询某订阅已下载的集数列表
   * 从 download_records 表中汇总：
   * 1. 关联该订阅 ID 的记录
   * 2. 同一 tmdbId + season 的记录（包含手动下载、其他订阅下载的）
   * 3. 无 tmdbId 但同 mediaTitle + season 的手动下载记录
   */
  async getDownloadedEpisodes(
    subscriptionId: string,
    tmdbId?: string,
    season?: number | null,
    mediaTitle?: string,
  ): Promise<Set<number>> {
    // 构建 OR 查询：订阅 ID / 同 tmdbId+season / 同 mediaTitle+season(手动下载)
    const orConditions: Record<string, unknown>[] = [{ subscriptionId }];
    if (tmdbId && season != null) {
      orConditions.push({ tmdbId: String(tmdbId), season: String(season) });
    }
    if (mediaTitle && season != null) {
      orConditions.push({
        season: String(season),
        tmdbId: null,
        mediaTitle,
      });
    }

    const records = await prisma.downloadRecord.findMany({
      where: {
        OR: orConditions,
        status: { not: "failed" },
      },
      select: { episode: true, episodes: true },
    });

    const downloaded = new Set<number>();
    for (const r of records) {
      // 优先使用 episodes 数组字段
      const eps = r.episodes as number[] | null;
      if (eps && eps.length > 0) {
        for (const e of eps) downloaded.add(e);
      } else if (r.episode) {
        // 向后兼容：旧的单集字段
        const ep = Number.parseInt(r.episode, 10);
        if (!Number.isNaN(ep)) downloaded.add(ep);
      }
    }
    return downloaded;
  }

  /**
   * 查询某订阅已下载的种子 hash 集合
   */
  async getDownloadedHashes(
    subscriptionId: string,
    tmdbId?: string,
    season?: number | null,
    mediaTitle?: string,
  ): Promise<Set<string>> {
    const orConditions: Record<string, unknown>[] = [{ subscriptionId }];
    if (tmdbId && season != null) {
      orConditions.push({ tmdbId: String(tmdbId), season: String(season) });
    }
    if (mediaTitle && season != null) {
      orConditions.push({
        season: String(season),
        tmdbId: null,
        mediaTitle,
      });
    }

    const records = await prisma.downloadRecord.findMany({
      where: {
        OR: orConditions,
        status: { not: "failed" },
      },
      select: { torrentHash: true },
    });

    const hashes = new Set<string>();
    for (const r of records) {
      if (r.torrentHash) hashes.add(r.torrentHash.toLowerCase());
    }
    return hashes;
  }

  /**
   * 获取订阅的集数进度摘要
   */
  async getEpisodeProgress(
    subscriptionId: string,
  ): Promise<{ downloadedEpisodes: number[]; totalEpisodes: number | null }> {
    const downloaded = await this.getDownloadedEpisodes(subscriptionId);
    const downloadedArr = [...downloaded].sort((a, b) => a - b);
    // totalEpisodes 需要从外层传入（TMDB 信息），这里只返回已下载数据
    return { downloadedEpisodes: downloadedArr, totalEpisodes: null };
  }

  /**
   * 执行单个订阅（搜索 → 过滤 → 匹配 → 下载）
   * @param id 订阅 ID
   * @param externalRunId 外部传入的 runId（可选，不传则自动生成）
   * @returns 是否成功下载了种子
   */
  async executeSubscription(
    id: string,
    externalRunId?: string,
  ): Promise<boolean> {
    const sub = await this.getById(id);
    if (!sub) return false;

    const run = createRunContext(id, externalRunId);
    await run.log("start", `开始执行订阅: ${sub.title}`, {
      tmdbId: sub.tmdbId,
      mediaType: sub.mediaType,
      season: sub.season,
    });

    try {
      // ===== 成人系列模式: 委托给专用服务 =====
      if (sub.subscriptionMode === "adult_series") {
        const result = await executeAdultSeriesSubscription(sub, run);
        await updateNextCheck(id, sub.intervalMinutes);
        await trimLogs(id);
        return result;
      }

      // ===== TV 剧集: 获取已下载状态 =====
      let downloadedEpisodes = new Set<number>();
      let downloadedHashes = new Set<string>();
      /** 本次需要搜索的缺失集数 */
      let missingEpisodes: number[] | null = null;

      /** TMDB 返回的该季有效集数列表 */
      let validEpisodes: number[] | null = null;
      /** 前季累计集数偏移量（用于绝对集数映射） */
      let seasonOffset: number | null = null;
      /** 种子集数映射表 */
      let episodeMap: Map<string, EpisodeMapping> | null = null;

      if (sub.mediaType === "tv") {
        downloadedEpisodes = await this.getDownloadedEpisodes(
          id,
          String(sub.tmdbId),
          sub.season,
          sub.title,
        );
        downloadedHashes = await this.getDownloadedHashes(
          id,
          String(sub.tmdbId),
          sub.season,
          sub.title,
        );

        if (downloadedEpisodes.size > 0) {
          await run.log(
            "start",
            `已下载集数: [${[...downloadedEpisodes].sort((a, b) => a - b).join(", ")}]`,
          );
        }

        // 从 TMDB 获取该季有效集数，用于过滤无效种子
        if (sub.season != null && sub.tmdbId != null) {
          validEpisodes = await this.fetchValidEpisodes(
            sub.tmdbId,
            sub.season,
            (msg) => run.log("start", msg),
          );
          if (validEpisodes) {
            await run.log(
              "start",
              `TMDB 该季有效集数: [${validEpisodes.join(", ")}] (共 ${validEpisodes.length} 集)`,
            );
          }
        }

        // 计算前季累计集数偏移量（用于绝对集数映射）
        // 条件：TMDB 集数从 1 开始（非海贼王那种 TMDB 自身绝对编号）且 season > 1
        if (
          validEpisodes &&
          sub.season != null &&
          sub.tmdbId != null &&
          sub.season > 1 &&
          Math.min(...validEpisodes) === 1
        ) {
          seasonOffset = await this.calculateSeasonOffset(
            sub.tmdbId,
            sub.season,
            (msg) => run.log("start", msg),
          );
        }

        // 计算缺失集：优先使用用户指定 → TMDB 有效集数 → 仅靠已下载集数
        if (sub.episodes && sub.episodes.length > 0) {
          // 用户手动指定了需要的集数
          missingEpisodes = sub.episodes.filter(
            (e) => !downloadedEpisodes.has(e),
          );
        } else if (validEpisodes) {
          // 使用 TMDB 集数列表减去已下载，自动算出缺失
          missingEpisodes = validEpisodes.filter(
            (e) => !downloadedEpisodes.has(e),
          );
        }

        if (missingEpisodes && missingEpisodes.length === 0) {
          await run.log("completed", "所有集数已下载完成，标记订阅为完成");
          await prisma.subscription.update({
            where: { id },
            data: { status: "completed", updatedAt: new Date() },
          });
          await updateNextCheck(id, sub.intervalMinutes);
          return false;
        }
        if (missingEpisodes) {
          await run.log("start", `缺失集数: [${missingEpisodes.join(", ")}]`);
        }
      }

      // 1. 构建搜索关键词
      const keywords = buildSearchKeywords(sub, missingEpisodes);
      await run.log("searching", `搜索关键词: ${keywords.join(" | ")}`);

      // 2. 搜索 PT 站点（多关键词搜索并去重）
      const PAGE_SIZE_THRESHOLD = 100;
      const torrents: PtTorrent[] = [];
      const seenTorrentKeys = new Set<string>();
      const resultCountBySite = new Map<string, number>();

      const searchPtSites = (keyword: string): Promise<PtTorrent[]> => {
        const siteIds = sub.siteIds ?? undefined;
        return siteIds && siteIds.length > 0
          ? searchService.searchPtSitesByIds(keyword, siteIds)
          : searchService.searchPtSites("system", keyword);
      };

      for (const keyword of keywords) {
        const results = await searchPtSites(keyword);

        for (const t of results) {
          const key = `${t.siteId}:${t.id}`;
          if (!seenTorrentKeys.has(key)) {
            seenTorrentKeys.add(key);
            torrents.push(t);
          }
          resultCountBySite.set(
            t.siteId,
            (resultCountBySite.get(t.siteId) ?? 0) + 1,
          );
        }
      }

      const isSearchTruncated = [...resultCountBySite.values()].some(
        (count) => count >= PAGE_SIZE_THRESHOLD,
      );

      await run.log("searching", `搜索到 ${torrents.length} 个种子`, {
        count: torrents.length,
        torrents: torrents.map((t) => ({
          title: t.title,
          size: t.size,
          seeders: t.seeders,
          leechers: t.leechers,
          siteId: t.siteId,
          siteName: t.siteName,
          detailsUrl: t.detailsUrl ?? null,
          downloadVolumeFactor: t.downloadVolumeFactor ?? null,
        })),
      });

      if (torrents.length === 0) {
        await run.log("completed", "未找到任何种子，本次结束");
        await updateNextCheck(id, sub.intervalMinutes);
        return false;
      }

      // 3. 应用过滤规则（多规则 AND 关系：串行过滤，每个种子必须通过所有规则）
      const filterIds = sub.filterIds ?? [];

      const applySubscriptionFilters = async (
        input: PtTorrent[],
      ): Promise<PtTorrent[]> => {
        let result = input;
        for (const fid of filterIds) {
          const filter = await subscriptionFilterService.getById(fid as string);
          if (filter) {
            const effectiveFilter =
              fid === filterIds[0] && sub.filterOverrides
                ? { ...filter, ...sub.filterOverrides }
                : filter;
            result = subscriptionFilterService.applyFilter(
              result,
              effectiveFilter,
            );
          }
        }
        return result;
      };

      let filtered = await applySubscriptionFilters(torrents);

      await run.log("filtering", `过滤后剩余 ${filtered.length} 个种子`, {
        count: filtered.length,
        torrents: filtered.map((t) => ({
          title: t.title,
          size: t.size,
          seeders: t.seeders,
          leechers: t.leechers,
          siteId: t.siteId,
          siteName: t.siteName,
          detailsUrl: t.detailsUrl ?? null,
          downloadVolumeFactor: t.downloadVolumeFactor ?? null,
        })),
      });

      if (filtered.length === 0) {
        await run.log("completed", "过滤后没有符合条件的种子");
        await updateNextCheck(id, sub.intervalMinutes);
        return false;
      }

      // 3.5 集数验证 + 绝对集数映射
      if (sub.mediaType === "tv" && validEpisodes) {
        const mapping = buildEpisodeMapping(
          filtered,
          validEpisodes,
          seasonOffset,
        );
        filtered = mapping.filtered;
        episodeMap = mapping.episodeMap;

        if (mapping.invalidCount > 0 || mapping.absoluteCount > 0) {
          const parts: string[] = [];
          if (mapping.invalidCount > 0) {
            parts.push(`过滤 ${mapping.invalidCount} 个无效集数种子`);
          }
          if (mapping.absoluteCount > 0) {
            parts.push(
              `映射 ${mapping.absoluteCount} 个绝对编号种子 (offset=${seasonOffset})`,
            );
          }
          await run.log(
            "filtering",
            `集数验证: ${parts.join("，")}，剩余 ${filtered.length} 个`,
            { count: filtered.length },
          );
        }
        if (filtered.length === 0) {
          await run.log("completed", "集数验证后没有有效种子");
          await updateNextCheck(id, sub.intervalMinutes);
          return false;
        }
      }

      // 3.6 搜索结果截断时，对未覆盖的缺失集逐集补充搜索
      if (
        sub.mediaType === "tv" &&
        isSearchTruncated &&
        missingEpisodes &&
        missingEpisodes.length > 0 &&
        sub.season != null
      ) {
        const coveredByTorrents = new Set<number>();
        let hasFullSeasonPack = false;

        for (const t of filtered) {
          const key = `${t.siteId}:${t.id}`;
          const mapping = episodeMap?.get(key);
          if (mapping) {
            if (mapping.mapped.length === 0) {
              hasFullSeasonPack = true;
              break;
            }
            for (const e of mapping.mapped) coveredByTorrents.add(e);
          } else {
            const parsed = parseMediaFilename(t.title);
            if (!parsed.episodes || parsed.episodes.length === 0) {
              hasFullSeasonPack = true;
              break;
            }
            for (const e of parsed.episodes) coveredByTorrents.add(e);
          }
        }

        if (!hasFullSeasonPack) {
          const uncoveredEps = missingEpisodes.filter(
            (e) => !coveredByTorrents.has(e) && !downloadedEpisodes.has(e),
          );

          if (uncoveredEps.length > 0) {
            const seasonStr = `S${String(sub.season).padStart(2, "0")}`;
            await run.log(
              "searching",
              `搜索结果可能截断，补充搜索 ${uncoveredEps.length} 个未覆盖集: [${uncoveredEps.join(", ")}]`,
            );

            for (const ep of uncoveredEps) {
              const epKeyword = `${sub.title} ${seasonStr}E${String(ep).padStart(2, "0")}`;
              try {
                const results = await searchPtSites(epKeyword);

                const newResults: PtTorrent[] = [];
                for (const t of results) {
                  const key = `${t.siteId}:${t.id}`;
                  if (!seenTorrentKeys.has(key)) {
                    seenTorrentKeys.add(key);
                    newResults.push(t);
                  }
                }

                if (newResults.length > 0) {
                  let extraFiltered =
                    await applySubscriptionFilters(newResults);

                  if (validEpisodes) {
                    const extraMapping = buildEpisodeMapping(
                      extraFiltered,
                      validEpisodes,
                      seasonOffset,
                    );
                    extraFiltered = extraMapping.filtered;
                    if (episodeMap) {
                      for (const [k, v] of extraMapping.episodeMap) {
                        episodeMap.set(k, v);
                      }
                    }
                  }

                  if (extraFiltered.length > 0) {
                    filtered = [...filtered, ...extraFiltered];
                    await run.log(
                      "searching",
                      `${seasonStr}E${String(ep).padStart(2, "0")}: 补充搜索新增 ${extraFiltered.length} 个种子`,
                    );
                  }
                }
              } catch (err) {
                await run.log(
                  "error",
                  `E${String(ep).padStart(2, "0")} 补充搜索失败: ${err instanceof Error ? err.message : "未知错误"}`,
                );
              }
              await new Promise((r) => setTimeout(r, 1000));
            }
          }
        }
      }

      // 4. 匹配逻辑
      let anyDownloaded = false;

      if (sub.mediaType === "tv") {
        // TV: 使用多种子贪心匹配，覆盖缺失集
        const selectedTorrents = matchTorrentsForTv(
          sub,
          filtered,
          downloadedEpisodes,
          downloadedHashes,
          validEpisodes,
          episodeMap,
        );

        if (selectedTorrents.length === 0) {
          await run.log("matching", "没有准确匹配的种子", {
            checked: filtered.length,
          });
          await run.log("completed", "本次未找到匹配种子");
          await updateNextCheck(id, sub.intervalMinutes);
          return false;
        }

        await run.log("matching", `匹配到 ${selectedTorrents.length} 个种子`, {
          torrents: selectedTorrents.map((s) => ({
            title: s.torrent.title,
            coveredEpisodes: s.coveredEpisodes,
          })),
        });

        // 5. 提交下载（按集数从小到大排序）
        selectedTorrents.sort((a, b) => {
          const aMin =
            a.coveredEpisodes.length > 0
              ? Math.min(...a.coveredEpisodes)
              : Number.MAX_SAFE_INTEGER;
          const bMin =
            b.coveredEpisodes.length > 0
              ? Math.min(...b.coveredEpisodes)
              : Number.MAX_SAFE_INTEGER;
          return aMin - bMin;
        });

        for (const selection of selectedTorrents) {
          const matched = selection.torrent;
          await run.log("downloading", `提交下载: ${matched.title}`, {
            torrentId: matched.id,
            torrentName: matched.title,
            coveredEpisodes: selection.coveredEpisodes,
            downloadUrl: matched.downloadUrl ? "[MASKED]" : "missing",
            detailsUrl: matched.detailsUrl ?? null,
          });

          try {
            // 按集数筛选文件：只下载需要的集数对应的文件
            const fileIndices = await this.getEpisodeAwareFileIndices(
              matched.siteId,
              matched.id,
              selection.originalEpisodes,
            );

            const record = await downloadManageService.submitDownload(
              {
                downloadUrl: matched.downloadUrl,
                torrentId: matched.id,
                torrentName: matched.title,
                size: matched.size,
                sizeBytes: matched.sizeBytes,
                ptSiteId: matched.siteId,
                ptSiteName: matched.siteName,
                seeders: matched.seeders,
                leechers: matched.leechers,
                contentType: sub.mediaType,
                tmdbId: String(sub.tmdbId),
                posterPath: sub.posterPath ?? undefined,
                mediaTitle: sub.title,
                mediaYear: sub.year ?? undefined,
                season: sub.season != null ? String(sub.season) : undefined,
                quality: matched.resolution ?? undefined,
                source: matched.source ?? undefined,
                codec: matched.videoCodec ?? undefined,
                downloadClientId: sub.downloadClientId ?? undefined,
                targetLibraryId: sub.targetLibraryId ?? undefined,
                subscriptionId: id,
                autoOrganize: true,
                isTrafficManage: false,
                selectedFileIndices: fileIndices,
              },
              undefined,
            );

            await run.log("downloading", `下载已提交，记录 ID: ${record.id}`, {
              recordId: record.id,
              torrentName: matched.title,
              coveredEpisodes: selection.coveredEpisodes,
            });

            // 记录文件结构
            await this.logFileStructure(
              run,
              matched.siteId,
              matched.id,
              selection.originalEpisodes,
            );

            // 将覆盖的集数标记为已下载（在内存中），避免后续种子重复覆盖
            for (const ep of selection.coveredEpisodes) {
              downloadedEpisodes.add(ep);
            }
            anyDownloaded = true;
          } catch (dlErr) {
            await run.log(
              "error",
              `下载 ${matched.title} 失败: ${dlErr instanceof Error ? dlErr.message : "未知错误"}`,
            );
          }
        }

        if (anyDownloaded) {
          // 发送通知
          notificationService
            .sendNotification(
              "subscription_download_started",
              {
                title: `订阅匹配: ${sub.title}`,
                message: `下载 ${selectedTorrents.length} 个种子`,
              },
              sub.createdBy,
            )
            .catch(() => {});

          // TV 自动完成检查（传入 TMDB 有效集数）
          await checkTvCompletion(id, sub, downloadedEpisodes, validEpisodes);
        }
      } else {
        // Movie: 原有逻辑，只选一个最佳种子
        const matched = matchTorrents(sub, filtered, downloadedHashes);

        if (!matched) {
          await run.log("matching", "没有准确匹配的种子", {
            checked: filtered.length,
          });
          await run.log("completed", "本次未找到匹配种子");
          await updateNextCheck(id, sub.intervalMinutes);
          return false;
        }

        await run.log("matching", `匹配到种子: ${matched.title}`, {
          torrentId: matched.id,
          torrentName: matched.title,
          siteId: matched.siteId,
          siteName: matched.siteName,
          size: matched.size,
          seeders: matched.seeders,
          detailsUrl: matched.detailsUrl ?? null,
        });

        // 5. 提交下载
        await run.log("downloading", `提交下载: ${matched.title}`, {
          torrentId: matched.id,
          torrentName: matched.title,
          downloadUrl: matched.downloadUrl ? "[MASKED]" : "missing",
          detailsUrl: matched.detailsUrl ?? null,
        });

        const record = await downloadManageService.submitDownload(
          {
            downloadUrl: matched.downloadUrl,
            torrentId: matched.id,
            torrentName: matched.title,
            size: matched.size,
            sizeBytes: matched.sizeBytes,
            ptSiteId: matched.siteId,
            ptSiteName: matched.siteName,
            seeders: matched.seeders,
            leechers: matched.leechers,
            contentType: sub.mediaType,
            tmdbId: String(sub.tmdbId),
            posterPath: sub.posterPath ?? undefined,
            mediaTitle: sub.title,
            mediaYear: sub.year ?? undefined,
            season: sub.season != null ? String(sub.season) : undefined,
            quality: matched.resolution ?? undefined,
            source: matched.source ?? undefined,
            codec: matched.videoCodec ?? undefined,
            downloadClientId: sub.downloadClientId ?? undefined,
            targetLibraryId: sub.targetLibraryId ?? undefined,
            subscriptionId: id,
            autoOrganize: true,
            isTrafficManage: false,
          },
          undefined,
        );

        // 记录文件结构
        await this.logFileStructure(run, matched.siteId, matched.id, []);

        await run.log("completed", `下载已提交，记录 ID: ${record.id}`, {
          recordId: record.id,
          torrentName: matched.title,
        });

        // 发送通知
        notificationService
          .sendNotification(
            "subscription_download_started",
            {
              title: `订阅匹配: ${sub.title}`,
              message: `种子: ${matched.title}\n来源: ${matched.siteName}`,
            },
            sub.createdBy,
          )
          .catch(() => {});

        // 对电影订阅，下载后标记为 completed
        await prisma.subscription.update({
          where: { id },
          data: { status: "completed", updatedAt: new Date() },
        });

        anyDownloaded = true;
      }

      if (anyDownloaded) {
        await run.log("completed", "本次执行完成，已提交下载");
      } else {
        await run.log("completed", "本次未成功下载任何种子");
      }

      await updateNextCheck(id, sub.intervalMinutes);
      await trimLogs(id);

      return anyDownloaded;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "订阅执行出错";
      await run.log("error", errorMsg);

      // 发送错误通知
      notificationService
        .sendNotification(
          "subscription_error",
          {
            title: `订阅执行失败: ${sub.title}`,
            message: errorMsg,
          },
          sub.createdBy,
        )
        .catch(() => {});

      await updateNextCheck(id, sub.intervalMinutes);
      return false;
    }
  }

  /**
   * 获取所有到期需要执行的订阅
   */
  async getDueSubscriptions(): Promise<Subscription[]> {
    const now = new Date();
    const rows = await prisma.subscription.findMany({
      where: {
        status: "active",
        nextCheckAt: { lte: now },
      },
      include: { filter: true, creator: true },
    });

    return rows.map((r) =>
      toSubscriptionOutput({
        ...r,
        filterName: r.filter?.name ?? null,
        createdByName: r.creator?.name ?? null,
      }),
    );
  }

  // ==================== 辅助方法 ====================

  /**
   * 记录种子文件结构到日志
   * 显示种子内所有文件及其选中/跳过状态
   */
  /**
   * 根据需要的集数获取应选中的文件索引
   * 返回 undefined 表示无法解析，让 submitDownload 走默认自动筛选
   */
  private async getEpisodeAwareFileIndices(
    siteId: string,
    torrentId: string,
    wantedEpisodes: number[],
  ): Promise<number[] | undefined> {
    if (wantedEpisodes.length === 0) return undefined;
    try {
      const preview = await downloadManageService.previewTorrentFiles(
        siteId,
        torrentId,
      );
      const filtered = selectTorrentFilesByEpisodes(
        preview.files,
        wantedEpisodes,
      );
      const indices = filtered.filter((f) => f.selected).map((f) => f.index);
      // 如果筛选后和原来一样，不需要指定
      const originalSelected = preview.files
        .filter((f) => f.selected)
        .map((f) => f.index);
      if (indices.length === originalSelected.length) return undefined;
      return indices;
    } catch {
      return undefined;
    }
  }

  private async logFileStructure(
    run: {
      log: (
        phase: SubscriptionLogEntry["phase"],
        message: string,
        details?: unknown,
      ) => Promise<void>;
    },
    siteId: string,
    torrentId: string,
    wantedEpisodes: number[],
  ): Promise<void> {
    try {
      const preview = await downloadManageService.previewTorrentFiles(
        siteId,
        torrentId,
      );

      // 按集数过滤文件选中状态
      const files =
        wantedEpisodes.length > 0
          ? selectTorrentFilesByEpisodes(preview.files, wantedEpisodes)
          : preview.files;
      const { totalSize, selectedSize } = {
        totalSize: preview.totalSize,
        selectedSize: files
          .filter((f) => f.selected)
          .reduce((sum, f) => sum + f.size, 0),
      };

      await run.log(
        "downloading",
        `文件结构 (${files.filter((f) => f.selected).length}/${files.length} 选中)`,
        {
          fileStructure: files.map((f) => ({
            path: f.path,
            size: f.size,
            selected: f.selected,
            fileType: f.fileType,
          })),
          totalSize,
          selectedSize,
        },
      );
    } catch {
      // 文件结构获取失败不影响主流程
    }
  }
}

export const subscriptionService = new SubscriptionService();
