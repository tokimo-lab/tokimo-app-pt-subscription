/**
 * 订阅匹配辅助函数（纯函数/无状态，提取自 SubscriptionService）
 * 包含：搜索关键词构建、集数映射、种子评分、匹配
 */
import type { PtTorrent, Subscription } from "@tokiomo/types";
import { prisma } from "../../db/client";
import { parseMediaFilename } from "../../lib/media-parser";
import { notificationService } from "../notification/notification.service";

/** 种子集数映射信息（用于绝对/相对集数兼容） */
export type EpisodeMapping = {
  /** 映射后的相对集数 */
  mapped: number[];
  /** 种子原始集数 */
  original: number[];
  /** 是否为绝对编号 */
  isAbsolute: boolean;
};

/**
 * 构建搜索关键词（支持多关键词）
 * 电影: ["{title} {year}"]
 * TV 全季: ["{title} S{season}"]（不含年份，避免跨年季搜不到）
 * TV 缺少少量集: ["{title} S{season}E{ep}", ...] + ["{title} S{season}"]
 *
 * TV 不加年份原因: 剧集首播年份与后续季年份可能不同（如 S01=2023, S02=2026），
 * 搜索时加年份会导致搜不到结果。年份改为在 scoreTorrent 中后处理打分。
 */
export function buildSearchKeywords(
  sub: Subscription,
  missingEpisodes: number[] | null,
): string[] {
  // 电影: 搜索时附带年份以精确匹配
  if (sub.mediaType !== "tv" || sub.season == null) {
    let base = sub.title;
    if (sub.year) {
      base += ` ${sub.year}`;
    }
    return [base];
  }

  // TV: 不加年份，仅用标题 + 季号搜索
  const base = sub.title;
  const seasonStr = `S${String(sub.season).padStart(2, "0")}`;
  const seasonKeyword = `${base} ${seasonStr}`;

  // TV: 如果缺失集数较少（<=3），先精确搜索每一集，再搜全季
  if (
    missingEpisodes &&
    missingEpisodes.length > 0 &&
    missingEpisodes.length <= 3
  ) {
    const episodeKeywords = missingEpisodes.map(
      (ep) => `${base} ${seasonStr}E${String(ep).padStart(2, "0")}`,
    );
    // 加上全季搜索，确保也能找到全季包
    return [...episodeKeywords, seasonKeyword];
  }

  return [seasonKeyword];
}

/**
 * 构建种子集数映射，将绝对编号种子映射为相对编号
 * - 相对编号种子（集数 ∈ validEpisodes）：直接保留
 * - 绝对编号种子（(集数 - offset) ∈ validEpisodes）：映射为相对集数
 * - 无效种子：两者都不满足，过滤掉
 */
export function buildEpisodeMapping(
  torrents: PtTorrent[],
  validEpisodes: number[],
  seasonOffset: number | null,
): {
  filtered: PtTorrent[];
  episodeMap: Map<string, EpisodeMapping>;
  absoluteCount: number;
  invalidCount: number;
} {
  const validSet = new Set(validEpisodes);
  const episodeMap = new Map<string, EpisodeMapping>();
  const filtered: PtTorrent[] = [];
  let absoluteCount = 0;
  let invalidCount = 0;

  for (const t of torrents) {
    const key = `${t.siteId}:${t.id}`;
    const parsed = parseMediaFilename(t.title);

    // 无集数（全季包）→ 保留
    if (!parsed.episodes || parsed.episodes.length === 0) {
      filtered.push(t);
      episodeMap.set(key, { mapped: [], original: [], isAbsolute: false });
      continue;
    }

    const eps = parsed.episodes;

    // 相对编号：所有集数 ∈ validEpisodes
    if (eps.every((e) => validSet.has(e))) {
      filtered.push(t);
      episodeMap.set(key, { mapped: eps, original: eps, isAbsolute: false });
      continue;
    }

    // 绝对编号：(集数 - offset) 全部 > 0 且 ∈ validEpisodes
    if (seasonOffset != null && seasonOffset > 0) {
      const remapped = eps.map((e) => e - seasonOffset);
      if (remapped.every((e) => e > 0 && validSet.has(e))) {
        filtered.push(t);
        episodeMap.set(key, {
          mapped: remapped,
          original: eps,
          isAbsolute: true,
        });
        absoluteCount++;
        continue;
      }
    }

    // 无效种子
    invalidCount++;
  }

  return { filtered, episodeMap, absoluteCount, invalidCount };
}

/**
 * 为种子打分
 * 0 = 不匹配, >0 = 越高越好
 * TV 模式下，排除已下载集数的种子，按缺失覆盖率加分
 */
export function scoreTorrent(
  sub: Subscription,
  torrent: PtTorrent,
  downloadedEpisodes: Set<number>,
  _downloadedHashes: Set<string>,
  overrideEpisodes?: number[] | null,
  isAbsolute?: boolean,
): number {
  let score = 1;

  const parsed = parseMediaFilename(torrent.title);
  // 使用映射后的集数（支持绝对编号重映射）
  const episodes = overrideEpisodes ?? parsed.episodes;

  // 标题匹配 (简单包含匹配)
  const titleLower = sub.title.toLowerCase();
  const torrentTitleLower = torrent.title.toLowerCase();
  const parsedTitleLower = (parsed.title ?? "").toLowerCase();

  if (
    !torrentTitleLower.includes(titleLower) &&
    !parsedTitleLower.includes(titleLower) &&
    !titleLower.includes(parsedTitleLower)
  ) {
    score *= 0.1;
  }

  // 年份匹配
  if (sub.year && parsed.year) {
    if (String(parsed.year) === sub.year) {
      score += 5;
    } else {
      score *= 0.5;
    }
  }

  // 剧集季号匹配
  if (sub.mediaType === "tv" && sub.season != null && parsed.season != null) {
    if (parsed.season === sub.season) {
      score += 10;
    } else {
      return 0; // 季号不匹配直接排除
    }
  }

  // ===== TV 集数智能匹配 =====
  if (sub.mediaType === "tv" && downloadedEpisodes.size > 0) {
    if (episodes && episodes.length > 0) {
      // 检查种子覆盖的集数是否全部已下载
      const newEpisodes = episodes.filter((e) => !downloadedEpisodes.has(e));
      if (newEpisodes.length === 0) {
        // 种子只包含已下载的集，排除
        return 0;
      }
      // 覆盖更多缺失集的种子得分更高
      score += newEpisodes.length * 3;
    }
    // 全季包（无 parsed.episodes）不在此排除，由后续逻辑决定
  }

  // 特定集数匹配（用户指定了需要的集数）
  if (sub.episodes && sub.episodes.length > 0) {
    if (episodes && episodes.length > 0) {
      const wantedSet = new Set(sub.episodes);
      const matchedEps = episodes.filter((e) => wantedSet.has(e));
      if (matchedEps.length > 0) {
        // 覆盖更多想要的集数 → 更高分
        score += matchedEps.length * 2;
      } else {
        // 不包含任何想要的集数 → 大幅降分
        score *= 0.1;
      }
    }
    // 全季包虽然不一定包含目标集，但仍可接受（可选择性下载）
  }

  // 做种人数加分 (越多越好)
  score += Math.min(torrent.seeders / 10, 5);

  // 免费加分
  if (torrent.downloadVolumeFactor === 0) {
    score += 3;
  }

  // 文件大小适中加分 (偏好 1-50GB)
  if (torrent.sizeBytes) {
    const gb = torrent.sizeBytes / 1e9;
    if (gb >= 1 && gb <= 50) score += 2;
  }

  // 相对编号种子优先（非绝对编号加分，使相对编号种子在评分中排在前面）
  if (sub.mediaType === "tv" && isAbsolute === false) {
    score += 5;
  }

  return score;
}

/**
 * 在过滤后的种子中匹配最佳种子（电影/通用）
 * 排除已下载的种子 hash
 */
export function matchTorrents(
  sub: Subscription,
  torrents: PtTorrent[],
  downloadedHashes: Set<string>,
): PtTorrent | null {
  const scored = torrents
    .map((t) => ({
      torrent: t,
      score: scoreTorrent(sub, t, new Set(), downloadedHashes),
    }))
    .filter((s) => s.score > 0)
    .sort((a, b) => b.score - a.score);

  return scored[0]?.torrent ?? null;
}

/**
 * TV 剧集多种子贪心匹配
 * 按评分排序后贪心选择，每个种子覆盖部分缺失集，直到所有缺失集覆盖或无候选
 * 支持绝对集数映射：通过 episodeMap 获取映射后的集数
 */
export function matchTorrentsForTv(
  sub: Subscription,
  torrents: PtTorrent[],
  downloadedEpisodes: Set<number>,
  downloadedHashes: Set<string>,
  validEpisodes?: number[] | null,
  episodeMap?: Map<string, EpisodeMapping> | null,
): Array<{
  torrent: PtTorrent;
  coveredEpisodes: number[];
  originalEpisodes: number[];
}> {
  const validEpisodeSet = validEpisodes ? new Set(validEpisodes) : null;

  const scored: Array<{
    torrent: PtTorrent;
    score: number;
    mappedEpisodes: number[] | null;
    originalEpisodes: number[] | null;
    isAbsolute: boolean;
  }> = [];

  for (const t of torrents) {
    const key = `${t.siteId}:${t.id}`;
    const mapping = episodeMap?.get(key);

    let mappedEps: number[] | null;
    let originalEps: number[] | null;
    let isAbsolute: boolean;

    if (mapping) {
      mappedEps = mapping.mapped.length > 0 ? mapping.mapped : null;
      originalEps = mapping.original.length > 0 ? mapping.original : null;
      isAbsolute = mapping.isAbsolute;
    } else {
      // 无映射表时回退解析
      const parsed = parseMediaFilename(t.title);
      mappedEps = parsed.episodes ?? null;
      originalEps = parsed.episodes ?? null;
      isAbsolute = false;
      if (validEpisodeSet && mappedEps && mappedEps.length > 0) {
        if (mappedEps.some((e) => !validEpisodeSet.has(e))) continue;
      }
    }

    const score = scoreTorrent(
      sub,
      t,
      downloadedEpisodes,
      downloadedHashes,
      mappedEps,
      isAbsolute,
    );
    if (score <= 0) continue;

    scored.push({
      torrent: t,
      score,
      mappedEpisodes: mappedEps,
      originalEpisodes: originalEps,
      isAbsolute,
    });
  }

  scored.sort((a, b) => {
    if (b.score !== a.score) return b.score - a.score;
    const aMin = a.mappedEpisodes?.length
      ? Math.min(...a.mappedEpisodes)
      : Number.MAX_SAFE_INTEGER;
    const bMin = b.mappedEpisodes?.length
      ? Math.min(...b.mappedEpisodes)
      : Number.MAX_SAFE_INTEGER;
    return aMin - bMin;
  });

  // 贪心选择
  const selected: Array<{
    torrent: PtTorrent;
    coveredEpisodes: number[];
    originalEpisodes: number[];
  }> = [];
  const coveredSoFar = new Set(downloadedEpisodes);

  const wantedEpisodes = sub.episodes ? new Set(sub.episodes) : validEpisodeSet;

  for (const item of scored) {
    // 使用映射后的集数计算覆盖
    let newEpisodes: number[];
    if (item.mappedEpisodes && item.mappedEpisodes.length > 0) {
      newEpisodes = item.mappedEpisodes.filter(
        (e) =>
          !coveredSoFar.has(e) && (!wantedEpisodes || wantedEpisodes.has(e)),
      );
    } else {
      if (wantedEpisodes) {
        newEpisodes = [...wantedEpisodes].filter((e) => !coveredSoFar.has(e));
      } else {
        newEpisodes = [-1];
      }
    }

    if (newEpisodes.length === 0) continue;

    // 计算对应的原始集数（用于文件选择）
    let origForSelection: number[];
    if (item.isAbsolute && item.mappedEpisodes && item.originalEpisodes) {
      const mappedToOrig = new Map<number, number>();
      for (let i = 0; i < item.mappedEpisodes.length; i++) {
        mappedToOrig.set(item.mappedEpisodes[i]!, item.originalEpisodes[i]!);
      }
      origForSelection = newEpisodes
        .filter((e) => e > 0)
        .map((e) => mappedToOrig.get(e) ?? e);
    } else {
      origForSelection = newEpisodes.filter((e) => e > 0);
    }

    selected.push({
      torrent: item.torrent,
      coveredEpisodes: newEpisodes.filter((e) => e > 0),
      originalEpisodes: origForSelection,
    });

    for (const e of newEpisodes) {
      if (e > 0) coveredSoFar.add(e);
    }

    if (newEpisodes.includes(-1)) break;

    if (wantedEpisodes) {
      const allCovered = [...wantedEpisodes].every((e) => coveredSoFar.has(e));
      if (allCovered) break;
    }

    const maxSelect = wantedEpisodes
      ? wantedEpisodes.size
      : (sub.maxDownloadsPerRun ?? 10);
    if (selected.length >= maxSelect) break;
  }

  return selected;
}

/**
 * TV 自动完成检查
 * 当所有指定集数（或 TMDB 总集数）都已下载时，自动标记为完成
 */
export async function checkTvCompletion(
  subscriptionId: string,
  sub: Subscription,
  downloadedEpisodes: Set<number>,
  validEpisodes?: number[] | null,
): Promise<void> {
  // 确定需要完成的集数列表：用户指定 > TMDB 有效集数
  const requiredEpisodes = sub.episodes?.length ? sub.episodes : validEpisodes;

  if (requiredEpisodes && requiredEpisodes.length > 0) {
    const allDone = requiredEpisodes.every((e) => downloadedEpisodes.has(e));
    if (allDone) {
      await prisma.subscription.update({
        where: { id: subscriptionId },
        data: { status: "completed", updatedAt: new Date() },
      });
      notificationService
        .sendNotification(
          "subscription_download_started",
          {
            title: `订阅完成: ${sub.title}`,
            message: `所有集数已下载完成 (共 ${requiredEpisodes.length} 集)`,
          },
          sub.createdBy,
        )
        .catch(() => {});
    }
  }
}

/**
 * 更新下次检查时间
 */
export async function updateNextCheck(
  id: string,
  intervalMinutes: number,
): Promise<void> {
  const next = new Date(Date.now() + intervalMinutes * 60 * 1000);
  await prisma.subscription.update({
    where: { id },
    data: {
      lastCheckedAt: new Date(),
      nextCheckAt: next,
      updatedAt: new Date(),
    },
  });
}
