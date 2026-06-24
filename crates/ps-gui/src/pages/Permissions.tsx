import { useEffect, useState } from "react";
import { api } from "../api";
import { useI18n } from "../i18n";
import type { PermissionsView, SystemInfoView } from "../types";

function StatusBadge({ granted }: { granted: boolean }) {
  const { t } = useI18n();
  return (
    <span className={`badge ${granted ? "success" : "danger"}`}>
      {granted ? t("status.granted") : t("status.missing")}
    </span>
  );
}

function guiPermissionLabels(platform: string, t: (key: string) => string) {
  if (platform === "windows") {
    return { title: t("permissions.uiAutomation"), desc: t("permissions.uiAutomationDesc") };
  }
  if (platform === "linux") {
    return { title: t("permissions.atSpi"), desc: t("permissions.atSpiDesc") };
  }
  return { title: t("permissions.accessibility"), desc: t("permissions.accessibilityDesc") };
}

export default function PermissionsPage() {
  const { t } = useI18n();
  const [info, setInfo] = useState<SystemInfoView | null>(null);
  const [logs, setLogs] = useState("");
  const [message, setMessage] = useState("");

  useEffect(() => {
    void refresh();
    void loadLogs();
  }, []);

  async function refresh() {
    const data = await api.getSystemInfo();
    setInfo(data);
  }

  async function loadLogs() {
    const content = await api.getRecentLogs(200);
    setLogs(content || t("permissions.noLogs"));
  }

  async function copyPath(path: string) {
    await api.copyText(path);
    setMessage(t("permissions.pathCopied"));
  }

  async function requestPermissions() {
    const perms = await api.promptForPermissions();
    setInfo((prev) =>
      prev
        ? {
            ...prev,
            permissions: perms,
          }
        : prev,
    );
    setMessage(t("permissions.promptShown"));
  }

  function renderPermissionCard(
    title: string,
    description: string,
    granted: boolean,
    openSettings: () => Promise<void>,
    perms: PermissionsView,
  ) {
    return (
      <div className="card permission-card">
        <div style={{ display: "flex", justifyContent: "space-between", gap: 12 }}>
          <div>
            <h2>{title}</h2>
            <p className="muted">{description}</p>
          </div>
          <StatusBadge granted={granted} />
        </div>
        <div className="path-box">{perms.executable_path}</div>
        <div className="btn-row">
          <button className="primary" onClick={() => void openSettings()}>
            {t("permissions.openSettings")}
          </button>
          <button className="secondary" onClick={() => void copyPath(perms.executable_path)}>
            {t("permissions.copyPath")}
          </button>
          <button className="secondary" onClick={() => void refresh()}>
            {t("common.refresh")}
          </button>
        </div>
      </div>
    );
  }

  const perms = info?.permissions;
  const status = info?.status;
  const guiLabels = guiPermissionLabels(perms?.platform ?? "macos", t);

  return (
    <>
      <header className="page-header">
        <div>
          <h1>{t("permissions.title")}</h1>
          <p>{t("permissions.subtitle")}</p>
          {perms ? <p className="muted">Platform: {perms.platform}</p> : null}
        </div>
        <button className="secondary" onClick={() => void requestPermissions()}>
          {t("permissions.request")}
        </button>
      </header>

      {perms ? (
        <div className="grid" style={{ marginBottom: 16 }}>
          {renderPermissionCard(
            guiLabels.title,
            guiLabels.desc,
            perms.gui_capture,
            api.openAccessibilitySettings,
            perms,
          )}
          {renderPermissionCard(
            t("permissions.inputMonitoring"),
            t("permissions.inputMonitoringDesc"),
            perms.input_monitoring,
            api.openInputMonitoringSettings,
            perms,
          )}
        </div>
      ) : null}

      <div className="grid grid-2">
        <div className="card">
          <h2>{t("permissions.dataDir")}</h2>
          <div className="path-box">{status?.data_dir ?? perms?.data_dir}</div>
          <div className="btn-row" style={{ marginTop: 12 }}>
            <button className="secondary" onClick={() => void api.openDataDir()}>
              {t("permissions.openFolder")}
            </button>
            <button
              className="secondary"
              onClick={() => void copyPath(status?.data_dir ?? perms?.data_dir ?? "")}
            >
              {t("permissions.copyPath")}
            </button>
          </div>
        </div>
        <div className="card">
          <h2>{t("permissions.daemonStatus")}</h2>
          <div className="meta-grid">
            <div className="meta-item">
              <label>{t("permissions.running")}</label>
              <div>{status?.running ? t("common.yes") : t("common.no")}</div>
            </div>
            <div className="meta-item">
              <label>{t("permissions.drafts")}</label>
              <div>{status?.draft_count ?? 0}</div>
            </div>
            <div className="meta-item">
              <label>{t("permissions.dbPath")}</label>
              <div style={{ fontSize: "0.82rem" }}>{status?.db_path}</div>
            </div>
            <div className="meta-item">
              <label>{t("permissions.dbSize")}</label>
              <div>{((status?.db_size_bytes ?? 0) / 1024) | 0} KB</div>
            </div>
          </div>
        </div>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <div className="page-header" style={{ marginBottom: 12 }}>
          <div>
            <h2 style={{ margin: 0 }}>{t("permissions.logs")}</h2>
            <p className="muted" style={{ margin: "6px 0 0" }}>
              {t("permissions.logsSubtitle")}
            </p>
          </div>
          <button className="secondary" onClick={() => void loadLogs()}>
            {t("permissions.reloadLogs")}
          </button>
        </div>
        <pre className="log-box">{logs}</pre>
      </div>

      {message ? <div className="toast">{message}</div> : null}
    </>
  );
}
