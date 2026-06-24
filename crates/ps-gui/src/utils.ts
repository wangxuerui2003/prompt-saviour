import type { Locale } from "./i18n";

function formatRelativeTime(
  iso: string | null,
  t: (key: string, params?: Record<string, string | number>) => string,
): string {
  if (!iso) return t("common.never");
  const date = new Date(iso);
  const diff = Date.now() - date.getTime();
  if (diff < 60_000) return t("common.justNow");
  if (diff < 3_600_000) return t("common.minAgo", { n: Math.floor(diff / 60_000) });
  if (diff < 86_400_000) return t("common.hrAgo", { n: Math.floor(diff / 3_600_000) });
  return date.toLocaleString();
}

function truncate(text: string, max = 120): string {
  if (text.length <= max) return text;
  return `${text.slice(0, max)}…`;
}

function formatDateTime(iso: string | null, locale: Locale): string {
  if (!iso) return "";
  return new Date(iso).toLocaleString(locale === "zh-CN" ? "zh-CN" : "en-US");
}

export { formatRelativeTime, truncate, formatDateTime };
