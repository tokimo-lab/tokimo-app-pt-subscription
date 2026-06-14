import { useWindowNav } from "@tokimo/sdk";
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
    <div className="relative flex h-full">
      <Sidebar
        active={section}
        onNavigate={(id) => replace(id)}
        collapsed={collapsed}
        onToggleCollapsed={() => setCollapsed(!collapsed)}
      />
      <div className="flex-1 min-w-0 overflow-auto p-4">
        <Page />
      </div>
    </div>
  );
}
