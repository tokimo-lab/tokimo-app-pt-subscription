import { useQuery } from "@tanstack/react-query";
import { Button, useDateFormat } from "@tokimo/ui";
import { RefreshCw } from "lucide-react";
import { subscriptionsApi } from "../api/client";
import { LogTerminal } from "./LogTerminal";

interface WindowHandle {
  id: string;
  metadata: Record<string, unknown>;
  close: () => void;
}

interface Props {
  win: WindowHandle;
}

export default function SubscriptionLogsWindow({ win }: Props) {
  const subId = win.metadata.subscriptionId as string;
  const subTitle = win.metadata.subscriptionTitle as string;

  const { formatLong } = useDateFormat();

  const debugQuery = useQuery({
    queryKey: ["subscription-debug", subId],
    queryFn: () => subscriptionsApi.getDebugInfo(subId),
  });

  const logsQuery = useQuery({
    queryKey: ["subscription-raw-logs", subId],
    queryFn: () => subscriptionsApi.getRawLogs(subId),
    refetchInterval: 5000,
  });

  const debug = debugQuery.data;
  const stats = debug
    ? {
        totalRuns: debug.totalRuns,
        successfulDownloads: debug.successfulDownloads,
        lastMatchedAt: debug.lastMatchedAt,
      }
    : null;

  return (
    <div className="flex flex-col h-full bg-transparent">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-base shrink-0">
        <div>
          <h3 className="font-medium text-fg-primary">{subTitle}</h3>
          {stats && (
            <div className="flex gap-3 text-xs text-fg-muted mt-0.5">
              <span>总执行: {stats.totalRuns}</span>
              <span>成功下载: {stats.successfulDownloads}</span>
              {stats.lastMatchedAt && (
                <span>
                  上次匹配: {formatLong(new Date(stats.lastMatchedAt))}
                </span>
              )}
            </div>
          )}
        </div>
        <Button
          icon={<RefreshCw size={14} />}
          onClick={() => {
            void debugQuery.refetch();
            void logsQuery.refetch();
          }}
          loading={debugQuery.isRefetching || logsQuery.isRefetching}
        />
      </div>

      {/* Log terminal */}
      <div className="flex-1 min-h-0">
        <LogTerminal content={logsQuery.data ?? ""} className="h-full" />
      </div>

      <div className="flex justify-end px-4 py-3 border-t border-base shrink-0">
        <Button onClick={win.close}>关闭</Button>
      </div>
    </div>
  );
}
