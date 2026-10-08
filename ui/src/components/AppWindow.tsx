import { useStandaloneDocumentScroll, useWindowNav } from "@tokimo/sdk";
import { useEffect, useState } from "react";
import DownloadClientsPage from "./DownloadClientsPage";
import PtSitesPage from "./PtSitesPage";
import SearchPage from "./SearchPage";
import { type SectionId, Sidebar } from "./Sidebar";
import SubscriptionsPage from "./SubscriptionsPage";

const pages: Record<SectionId, React.FC> = {
  subscriptions: SubscriptionsPage,
  "download-clients": DownloadClientsPage,
  "pt-sites": PtSitesPage,
  search: SearchPage,
};

const DEFAULT_SECTION: SectionId = "subscriptions";

export function AppWindow() {
  const documentScroll = useStandaloneDocumentScroll();
  const { route, replace } = useWindowNav();
  const [collapsed, setCollapsed] = useState(false);

  const section: SectionId =
    (route as SectionId) in pages ? (route as SectionId) : DEFAULT_SECTION;

  // Initialize route if empty
  useEffect(() => {
    if (!route || !(route in pages)) {
      replace(DEFAULT_SECTION);
    }
  }, [route, replace]);

  const Page = pages[section];
  return (
    <div
      className={`relative flex ${documentScroll ? "min-h-dvh flex-col" : "h-full"}`}
    >
      <Sidebar
        active={section}
        onNavigate={(id) => replace(id)}
        collapsed={collapsed}
        onToggleCollapsed={() => setCollapsed(!collapsed)}
      />
      <div
        className={`app-safe-area flex-1 min-w-0 bg-surface-base [--app-safe-area-padding:1rem] ${documentScroll ? "overflow-visible" : "overflow-auto"}`}
      >
        <Page />
      </div>
    </div>
  );
}
