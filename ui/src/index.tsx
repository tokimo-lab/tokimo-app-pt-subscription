/**
 * PT 订阅管理 app — 管理 PT 站点订阅、RSS 源、自动下载规则。
 */

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { type Dispose, defineApp, RuntimeProvider } from "@tokimo/sdk";
import {
  ConfigProvider,
  ToastProvider,
  enUS as uiEnUS,
  zhCN as uiZhCN,
} from "@tokimo/ui";
import { StrictMode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import { AppWindow } from "./components/AppWindow";
import i18n, { SUPPORTED_LOCALES } from "./i18n";
import "./index.css";

export default defineApp({
  id: "pt-subscription",
  manifest: {
    id: "pt-subscription",
    appName: "PT 订阅管理",
    icon: "Rss",
    image: "icon.png",
    color: "#3b82f6",
    windowType: "pt-subscription",
    defaultSize: { width: 1080, height: 660 },
    category: "app",
  },
  mount(container, ctx): Dispose {
    const applyLocale = (raw: string) => {
      const target = SUPPORTED_LOCALES.includes(raw) ? raw : "en-US";
      if (i18n.language !== target) {
        void i18n.changeLanguage(target);
      }
    };

    applyLocale(ctx.locale);
    const unsubLocale = ctx.shell.subscribeLocale(applyLocale);

    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: 1, staleTime: 30_000 } },
    });
    const uiLocale = ctx.locale.startsWith("zh") ? uiZhCN : uiEnUS;
    const root: Root = createRoot(container);

    root.render(
      <StrictMode>
        <I18nextProvider i18n={i18n}>
          <RuntimeProvider value={ctx}>
            <QueryClientProvider client={queryClient}>
              <ConfigProvider
                locale={uiLocale}
                dateFormat={{
                  defaultLong: "YYYY-MM-DD HH:mm:ss",
                  defaultDate: "YYYY-MM-DD",
                  defaultTime: "HH:mm:ss",
                  storage: "none",
                }}
              >
                <ToastProvider>
                  <AppWindow />
                </ToastProvider>
              </ConfigProvider>
            </QueryClientProvider>
          </RuntimeProvider>
        </I18nextProvider>
      </StrictMode>,
    );

    return () => {
      unsubLocale();
      root.unmount();
    };
  },
});
