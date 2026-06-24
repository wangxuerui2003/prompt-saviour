import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "../api";
import { en, type Messages } from "./en";
import { zhCN } from "./zh-CN";

export type Locale = "en" | "zh-CN";
export type LocaleSetting = "system" | Locale;

const dictionaries: Record<Locale, Messages> = {
  en,
  "zh-CN": zhCN,
};

function detectSystemLocale(): Locale {
  if (typeof navigator !== "undefined" && navigator.language.toLowerCase().startsWith("zh")) {
    return "zh-CN";
  }
  return "en";
}

export function resolveLocale(setting: string | undefined | null): Locale {
  if (setting === "zh-CN" || setting === "en") return setting;
  return detectSystemLocale();
}

function lookup(obj: unknown, path: string): string | undefined {
  const value = path.split(".").reduce<unknown>((acc, key) => {
    if (acc && typeof acc === "object" && key in (acc as Record<string, unknown>)) {
      return (acc as Record<string, unknown>)[key];
    }
    return undefined;
  }, obj);
  return typeof value === "string" ? value : undefined;
}

function interpolate(template: string, params?: Record<string, string | number>): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (_, key: string) => String(params[key] ?? `{${key}}`));
}

interface I18nContextValue {
  locale: Locale;
  localeSetting: LocaleSetting;
  setLocaleSetting: (value: LocaleSetting) => Promise<void>;
  t: (key: string, params?: Record<string, string | number>) => string;
  messages: Messages;
}

const I18nContext = createContext<I18nContextValue | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [localeSetting, setLocaleSettingState] = useState<LocaleSetting>("system");
  const locale = useMemo(() => resolveLocale(localeSetting), [localeSetting]);

  useEffect(() => {
    void api.getConfig().then((config) => {
      setLocaleSettingState((config.ui_language as LocaleSetting) || "system");
    });
    const unlisten = listen<string>("ui-language-changed", (event) => {
      setLocaleSettingState((event.payload as LocaleSetting) || "system");
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  async function setLocaleSetting(value: LocaleSetting) {
    const config = await api.getConfig();
    config.ui_language = value;
    await api.saveConfig(config);
    setLocaleSettingState(value);
  }

  const value = useMemo<I18nContextValue>(() => {
    const messages = dictionaries[locale];
    return {
      locale,
      localeSetting,
      setLocaleSetting,
      messages,
      t: (key, params) => interpolate(lookup(messages, key) ?? lookup(en, key) ?? key, params),
    };
  }, [locale, localeSetting]);

  useEffect(() => {
    document.documentElement.lang = locale === "zh-CN" ? "zh-CN" : "en";
  }, [locale]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n must be used within I18nProvider");
  return ctx;
}
