import { AppSidebar, type AppSidebarSection } from "@tokimo/ui";
import { Download, Globe, Rss, Search } from "lucide-react";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";

export type SectionId =
  | "subscriptions"
  | "download-clients"
  | "pt-sites"
  | "search";

interface SidebarProps {
  active: SectionId;
  onNavigate: (id: SectionId) => void;
  collapsed: boolean;
  onToggleCollapsed: () => void;
}

export function Sidebar({
  active,
  onNavigate,
  collapsed,
  onToggleCollapsed,
}: SidebarProps) {
  const { t } = useTranslation();

  const sections: AppSidebarSection[] = useMemo(
    () => [
      {
        items: [
          {
            key: "subscriptions",
            icon: <Rss className="w-4 h-4" />,
            label: t("sidebar.subscriptions"),
          },
          {
            key: "download-clients",
            icon: <Download className="w-4 h-4" />,
            label: t("sidebar.downloadClients"),
          },
          {
            key: "pt-sites",
            icon: <Globe className="w-4 h-4" />,
            label: t("sidebar.ptSites"),
          },
          {
            key: "search",
            icon: <Search className="w-4 h-4" />,
            label: t("sidebar.search"),
          },
        ],
      },
    ],
    [t],
  );

  return (
    <AppSidebar
      sections={sections}
      activeKey={active}
      onSelect={(key) => onNavigate(key as SectionId)}
      collapsed={collapsed}
      onToggleCollapsed={onToggleCollapsed}
      width={188}
    />
  );
}
