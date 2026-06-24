import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { api } from "../api";
import { useI18n } from "../i18n";
import type { CurrentPromptUpdate, DaemonStatus, SessionContext } from "../types";
import { formatRelativeTime, truncate } from "../utils";

export default function DashboardPage() {
  const { t } = useI18n();
  const navigate = useNavigate();
  const [status, setStatus] = useState<DaemonStatus | null>(null);
  const [current, setCurrent] = useState<CurrentPromptUpdate | null>(null);
  const [session, setSession] = useState<SessionContext | null>(null);
  const [message, setMessage] = useState("");

  useEffect(() => {
    void refresh();
    const timer = window.setInterval(() => void refresh(), 1000);
    return () => window.clearInterval(timer);
  }, []);

  async function refresh() {
    const [daemonStatus, prompt, frontmost] = await Promise.all([
      api.getDaemonStatus(),
      api.getCurrentPrompt(),
      api.getFrontmostSession(),
    ]);
    setStatus(daemonStatus);
    setCurrent(prompt);
    setSession(frontmost);
  }

  async function togglePause() {
    const paused = !status?.paused;
    await api.setCapturePaused(!!paused);
    setMessage(paused ? t("dashboard.protectionPaused") : t("dashboard.protectionResumed"));
    await refresh();
  }

  async function copyLatest() {
    const draft = await api.recoverDraft();
    setMessage(t("dashboard.copiedDraft", { id: draft.id }));
  }

  const protectionBadge = !status?.running
    ? { label: t("status.stopped"), className: "danger" }
    : status.paused
      ? { label: t("status.paused"), className: "warning" }
      : { label: t("status.running"), className: "success" };

  const preview = current?.snapshot?.content ?? "";
  const charCount = current?.char_count ?? 0;

  return (
    <>
      <header className="page-header">
        <div>
          <h1>{t("dashboard.title")}</h1>
          <p>{t("dashboard.subtitle")}</p>
        </div>
        <span className={`badge ${protectionBadge.className}`}>{protectionBadge.label}</span>
      </header>

      <div className="grid grid-3">
        <div className="card">
          <h3>{t("dashboard.protection")}</h3>
          <div className="stat-value">{protectionBadge.label}</div>
          <p className="muted">
            Daemon {status?.running ? t("status.active") : t("status.inactive")}
          </p>
        </div>
        <div className="card">
          <h3>{t("dashboard.draftsSaved")}</h3>
          <div className="stat-value">{status?.draft_count ?? 0}</div>
          <p className="muted">
            {t("dashboard.dbSize", { size: ((status?.db_size_bytes ?? 0) / 1024) | 0 })}
          </p>
        </div>
        <div className="card">
          <h3>{t("dashboard.lastSaved")}</h3>
          <div className="stat-value" style={{ fontSize: "1.1rem" }}>
            {formatRelativeTime(current?.last_saved_at ?? null, t)}
          </div>
          <p className="muted">
            {current?.persisted ? t("dashboard.persisted") : t("dashboard.waitingDebounce")}
          </p>
        </div>
      </div>

      <div className="grid" style={{ marginTop: 16 }}>
        <div className="card">
          <h2>{t("dashboard.foregroundApp")}</h2>
          {session ? (
            <div className="meta-grid">
              <div className="meta-item">
                <label>{t("dashboard.app")}</label>
                <div>{session.app_name}</div>
              </div>
              <div className="meta-item">
                <label>{t("dashboard.bundle")}</label>
                <div>{session.bundle_id}</div>
              </div>
              <div className="meta-item">
                <label>{t("dashboard.window")}</label>
                <div>{session.window_title || t("common.dash")}</div>
              </div>
              <div className="meta-item">
                <label>PID</label>
                <div>{session.pid}</div>
              </div>
            </div>
          ) : (
            <p className="muted">{t("dashboard.noSession")}</p>
          )}
        </div>

        <div className="card">
          <h2>{t("dashboard.previewTitle")}</h2>
          {preview ? (
            <>
              <p>{truncate(preview)}</p>
              <p className="muted">
                {charCount} {t("common.chars")}
              </p>
            </>
          ) : (
            <p className="muted">{t("dashboard.waitingInput")}</p>
          )}
          <div className="btn-row" style={{ marginTop: 16 }}>
            <button className="primary" onClick={() => void togglePause()}>
              {status?.paused ? t("dashboard.resumeProtection") : t("dashboard.pauseProtection")}
            </button>
            <button className="secondary" onClick={() => navigate("/history")}>
              {t("dashboard.openHistory")}
            </button>
            <button className="secondary" onClick={() => void copyLatest()}>
              {t("dashboard.copyLatest")}
            </button>
          </div>
        </div>
      </div>

      {message ? <div className="toast">{message}</div> : null}
    </>
  );
}
