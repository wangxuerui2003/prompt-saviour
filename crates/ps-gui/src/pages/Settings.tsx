import { useEffect, useState } from "react";
import { api } from "../api";
import { useI18n, type LocaleSetting } from "../i18n";
import type { AppConfig } from "../types";

export default function SettingsPage() {
  const { t, localeSetting, setLocaleSetting } = useI18n();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [message, setMessage] = useState("");

  useEffect(() => {
    void api.getConfig().then(setConfig);
  }, []);

  function update<K extends keyof AppConfig>(key: K, value: AppConfig[K]) {
    setConfig((prev) => (prev ? { ...prev, [key]: value } : prev));
  }

  async function save() {
    if (!config) return;
    const next = { ...config, ui_language: localeSetting };
    await api.saveConfig(next);
    setConfig(next);
    setMessage(t("settings.saved"));
  }

  async function onLanguageChange(value: LocaleSetting) {
    setConfig((prev) => (prev ? { ...prev, ui_language: value } : prev));
    await setLocaleSetting(value);
  }

  if (!config) {
    return <div className="card">{t("common.loading")}</div>;
  }

  return (
    <>
      <header className="page-header">
        <div>
          <h1>{t("settings.title")}</h1>
          <p>{t("settings.subtitle")}</p>
        </div>
        <button className="primary" onClick={() => void save()}>
          {t("common.save")}
        </button>
      </header>

      <div className="card form-grid">
        <div className="form-row">
          <label>{t("settings.language")}</label>
          <select
            value={localeSetting}
            onChange={(e) => void onLanguageChange(e.target.value as LocaleSetting)}
          >
            <option value="system">{t("settings.langSystem")}</option>
            <option value="en">{t("settings.langEn")}</option>
            <option value="zh-CN">{t("settings.langZh")}</option>
          </select>
        </div>
        <div className="form-row">
          <label>{t("settings.capturePaused")}</label>
          <input
            type="checkbox"
            checked={config.capture_paused}
            onChange={(e) => update("capture_paused", e.target.checked)}
          />
        </div>
        <div className="form-row">
          <label>{t("settings.launchAtLogin")}</label>
          <input
            type="checkbox"
            checked={config.launch_at_login}
            onChange={(e) => update("launch_at_login", e.target.checked)}
          />
        </div>
        <div className="form-row">
          <label>{t("settings.axPoll")}</label>
          <input
            type="number"
            value={config.ax_poll_ms}
            onChange={(e) => update("ax_poll_ms", Number(e.target.value))}
          />
        </div>
        <div className="form-row">
          <label>{t("settings.debounce")}</label>
          <input
            type="number"
            value={config.debounce_ms}
            onChange={(e) => update("debounce_ms", Number(e.target.value))}
          />
        </div>
        <div className="form-row">
          <label>{t("settings.retention")}</label>
          <input
            type="number"
            value={config.retention_days}
            onChange={(e) => update("retention_days", Number(e.target.value))}
          />
        </div>
        <div className="form-row">
          <label>{t("settings.maxDrafts")}</label>
          <input
            type="number"
            value={config.max_drafts}
            onChange={(e) => update("max_drafts", Number(e.target.value))}
          />
        </div>
        <div className="form-row">
          <label>{t("settings.globalHotkey")}</label>
          <input
            value={config.global_hotkey}
            onChange={(e) => update("global_hotkey", e.target.value)}
          />
        </div>
        <div className="form-row">
          <label>{t("settings.crashToast")}</label>
          <input
            type="checkbox"
            checked={config.crash_toast_enabled}
            onChange={(e) => update("crash_toast_enabled", e.target.checked)}
          />
        </div>
        <div className="form-row">
          <label>{t("settings.recoverAction")}</label>
          <select
            value={config.recover_action}
            onChange={(e) => update("recover_action", e.target.value)}
          >
            <option value="clipboard">{t("settings.recoverClipboard")}</option>
            <option value="paste">{t("settings.recoverPaste")}</option>
          </select>
        </div>
        <div className="form-row">
          <label>{t("settings.excludedBundles")}</label>
          <textarea
            rows={4}
            value={config.excluded_bundle_ids.join("\n")}
            onChange={(e) =>
              update(
                "excluded_bundle_ids",
                e.target.value
                  .split("\n")
                  .map((s) => s.trim())
                  .filter(Boolean),
              )
            }
          />
        </div>
      </div>

      {message ? <div className="toast">{message}</div> : null}
    </>
  );
}
