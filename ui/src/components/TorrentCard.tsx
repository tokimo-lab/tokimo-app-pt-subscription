import {
  ArrowDownOutlined,
  ArrowUpOutlined,
  Button,
  CheckCircleOutlined,
  ClockCircleOutlined,
  DownloadOutlined,
  Image,
  LinkOutlined,
  StarOutlined,
  Tag,
  Tooltip,
} from "@tokimo/ui";
import type { PtSearchResultWithSite } from "../api/client";
import {
  formatDate,
  getCategoryColor,
  getCategoryName,
  getDownloadFactor,
  getUploadFactor,
  parseTitleTags,
} from "./search-utils";

export const getDiscountTags = (torrent: PtSearchResultWithSite) => {
  const tags: React.ReactNode[] = [];
  const dlFactor = getDownloadFactor(torrent);
  const ulFactor = getUploadFactor(torrent);

  if (dlFactor === 0) {
    tags.push(
      <Tag key="free" color="green" className="font-bold">
        免费
      </Tag>,
    );
  } else if (dlFactor === 0.5) {
    tags.push(
      <Tag key="half" color="cyan">
        50%
      </Tag>,
    );
  } else if (dlFactor === 0.3) {
    tags.push(
      <Tag key="70" color="blue">
        30%
      </Tag>,
    );
  }

  if (ulFactor > 1) {
    tags.push(
      <Tag key="upload" color="orange">
        {ulFactor}x↑
      </Tag>,
    );
  }

  return tags;
};

/** Parse tags from title — ignore API values (may be category IDs). */
export const getMediaTags = (torrent: PtSearchResultWithSite) => {
  const tags: React.ReactNode[] = [];
  const parsed = parseTitleTags(torrent.title, torrent.subtitle);

  if (parsed.resolution) {
    // 2160p 金色 > 1080p 绿色 > 720p 蓝色 > 480p 默认
    const color = parsed.resolution.includes("2160")
      ? "gold"
      : parsed.resolution.includes("1080")
        ? "green"
        : parsed.resolution.includes("720")
          ? "blue"
          : "default";
    tags.push(
      <Tag key="res" color={color}>
        {parsed.resolution}
      </Tag>,
    );
  }

  if (parsed.videoCodec) {
    // AV1 金色 > H265/x265 绿色 > H264/x264 蓝色 > VC-1 默认
    const color =
      parsed.videoCodec === "AV1"
        ? "gold"
        : parsed.videoCodec.includes("265")
          ? "green"
          : parsed.videoCodec.includes("264")
            ? "blue"
            : "default";
    tags.push(
      <Tag key="video" color={color}>
        {parsed.videoCodec}
      </Tag>,
    );
  }

  if (parsed.audioCodec) {
    // Atmos 金色 > 无损(DTS-HD/TrueHD) 绿色 > 好(DTS/FLAC) 蓝色 > 标准 默认
    const color =
      parsed.audioCodec === "Atmos"
        ? "gold"
        : parsed.audioCodec.includes("HD") || parsed.audioCodec === "TrueHD"
          ? "green"
          : parsed.audioCodec === "DTS" || parsed.audioCodec === "FLAC"
            ? "blue"
            : "default";
    tags.push(
      <Tag key="audio" color={color}>
        {parsed.audioCodec}
      </Tag>,
    );
  }

  if (parsed.source) {
    // Remux 金色 > BluRay 绿色 > WEB 蓝色 > 其他 默认
    const color =
      parsed.source === "Remux"
        ? "gold"
        : parsed.source === "BluRay"
          ? "green"
          : parsed.source.startsWith("WEB")
            ? "blue"
            : "default";
    tags.push(
      <Tag key="source" color={color}>
        {parsed.source}
      </Tag>,
    );
  }

  return tags;
};

export interface TorrentCardBodyProps {
  torrent: PtSearchResultWithSite;
}

export function TorrentCardBody({ torrent }: TorrentCardBodyProps) {
  const discountTags = getDiscountTags(torrent);
  const mediaTags = getMediaTags(torrent);

  return (
    <div className="flex items-start gap-3">
      {torrent.posterUrl ? (
        <div className="flex-shrink-0">
          <Image
            src={torrent.posterUrl}
            alt={torrent.title}
            width={60}
            height={85}
            className="object-cover rounded"
            preview={false}
          />
        </div>
      ) : null}

      <div className="flex-1 min-w-0">
        <Tooltip title={torrent.title}>
          <span className="block text-sm leading-tight line-clamp-2 font-semibold">
            {torrent.title}
          </span>
        </Tooltip>
        {torrent.subtitle && (
          <Tooltip title={torrent.subtitle}>
            <span className="block text-xs mt-0.5 line-clamp-1 text-[rgba(100,100,100,0.85)] dark:text-[rgba(180,180,180,0.6)]">
              {torrent.subtitle}
            </span>
          </Tooltip>
        )}

        <div className="mt-2 flex flex-wrap gap-1">
          {discountTags}
          {mediaTags}
          {torrent.imdbUrl && (
            <a
              href={torrent.imdbUrl}
              target="_blank"
              rel="noreferrer"
              onClick={(e) => e.stopPropagation()}
            >
              <Tag color="gold">
                <StarOutlined className="mr-0.5" />
                IMDb{torrent.imdbRating ? ` ${torrent.imdbRating}` : ""}
              </Tag>
            </a>
          )}
          {torrent.doubanUrl && (
            <a
              href={torrent.doubanUrl}
              target="_blank"
              rel="noreferrer"
              onClick={(e) => e.stopPropagation()}
            >
              <Tag color="green">
                豆瓣{torrent.doubanRating ? ` ${torrent.doubanRating}` : ""}
              </Tag>
            </a>
          )}
        </div>

        <div className="mt-2 flex flex-wrap items-center gap-1 text-xs">
          <Tag color="blue" className="mr-0">
            {torrent.siteName}
          </Tag>
          <Tag color={getCategoryColor(torrent.category)} className="mr-0">
            {torrent.categoryName || getCategoryName(torrent.category)}
          </Tag>
          <span className="text-fg-muted">{torrent.size}</span>
          {torrent.uploadTime && (
            <Tooltip title={torrent.uploadTime}>
              <span className="flex items-center gap-0.5 text-fg-muted">
                <ClockCircleOutlined /> {formatDate(torrent.uploadTime)}
              </span>
            </Tooltip>
          )}
          <Tooltip title="做种">
            <span className="text-green-600 flex items-center gap-0.5">
              <ArrowUpOutlined /> {torrent.seeders}
            </span>
          </Tooltip>
          <Tooltip title="下载">
            <span className="text-red-500 flex items-center gap-0.5">
              <ArrowDownOutlined /> {torrent.leechers}
            </span>
          </Tooltip>
          {torrent.grabs !== undefined && (
            <Tooltip title="完成">
              <span className="hidden lg:inline-flex items-center gap-0.5 text-fg-muted">
                <CheckCircleOutlined /> {torrent.grabs}
              </span>
            </Tooltip>
          )}
          {torrent.detailUrl && (
            <Tooltip title="详情">
              <a
                href={torrent.detailUrl}
                target="_blank"
                rel="noreferrer"
                onClick={(e) => e.stopPropagation()}
              >
                <LinkOutlined />
              </a>
            </Tooltip>
          )}
        </div>
      </div>
    </div>
  );
}

interface TorrentCardProps {
  torrent: PtSearchResultWithSite;
  onDownload: (t: PtSearchResultWithSite) => void;
}

const TorrentCard = ({ torrent, onDownload }: TorrentCardProps) => {
  return (
    <div className="px-3 py-3.5 hover:bg-black/[0.03] dark:hover:bg-white/[0.05] transition-colors">
      <div className="flex items-center gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-start gap-2">
            <div className="flex-1 min-w-0">
              <TorrentCardBody torrent={torrent} />
            </div>
            <Button
              variant="text"
              icon={<DownloadOutlined />}
              onClick={() => onDownload(torrent)}
              className="flex-shrink-0 mt-0.5"
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default TorrentCard;
