import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { api } from "../api";
import { useI18n } from "../i18n";
import type { CurrentPromptUpdate } from "../types";
import { formatRelativeTime } from "../utils";

export default function CurrentPromptPage() {
  const { t } = useI18n();
  const [current, setCurrent] = useState<CurrentPromptUpdate | null>(null);
  const [message, setMessage] = useState("");

  useEffect(() => {
    void api.getCurrentPrompt().then(setCurrent);
    const unlisten = listen<CurrentPromptUpdate>("current-prompt-update", (event) => {
      setCurrent(event.payload);
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  const snapshot = current?.snapshot;
  const content = snapshot?.content ?? "";
  const minChars = 8;

  async function copyPrompt() {
    if (!content) return;
    await api.copyText(content);
    setMessage(t("current.copied"));
  }

  async function exportTxt() {
    if (!content) return;
    await api.copyText(content);
    setMessage(t("current.exported"));
  }

  async function pinCurrent() {
    const drafts = await api.listDrafts(1);
    if (!drafts[0]) {
      setMessage(t("current.noDraftPin"));
      return;
    }
    await api.pinDraft(drafts[0].id, true);
    setMessage(t("current.pinned", { id: drafts[0].id }));
  }

  async function markSubmitted() {
    if (!snapshot) return;
    await api.markSlotSubmitted(snapshot.slot.key);
    setMessage(t("current.markedSubmitted"));
  }

  const emptyMessage = !snapshot
    ? t("current.waiting")
    : content.trim().length < minChars
      ? t("current.minChars", { n: minChars })
      : null;

  const confidenceKey = `status.${current?.confidence ?? "none"}`;

  return (
    <>
      <header className="page-header">
        <div>
          <h1>{t("current.title")}</h1>
          <p>{t("current.subtitle")}</p>
        </div>
        {current?.paused ? <span className="badge warning">{t("status.paused")}</span> : null}
      </header>

      <div className="meta-grid" style={{ marginBottom: 16 }}>
        <div className="meta-item">
          <label>{t("current.characters")}</label>
          <div>{current?.char_count ?? 0}</div>
        </div>
        <div className="meta-item">
          <label>{t("current.lines")}</label>
          <div>{current?.line_count ?? 0}</div>
        </div>
        <div className="meta-item">
          <label>{t("current.source")}</label>
          <div>{current?.source ?? t("common.dash")}</div>
        </div>
        <div className="meta-item">
          <label>{t("current.confidence")}</label>
          <div>{t(confidenceKey)}</div>
        </div>
      </div>

      <div className="card" style={{ marginBottom: 16 }}>
        <div className="meta-grid">
          <div className="meta-item">
            <label>{t("dashboard.app")}</label>
            <div>{snapshot?.slot.context.app_name ?? t("common.dash")}</div>
          </div>
          <div className="meta-item">
            <label>{t("dashboard.window")}</label>
            <div>{snapshot?.slot.context.window_title || t("common.dash")}</div>
          </div>
          <div className="meta-item">
            <label>{t("current.updated")}</label>
            <div>{formatRelativeTime(snapshot?.captured_at ?? null, t)}</div>
          </div>
          <div className="meta-item">
            <label>{t("current.persisted")}</label>
            <div>{current?.persisted ? t("current.savedLocally") : t("current.inDebounce")}</div>
          </div>
        </div>
      </div>

      {emptyMessage ? (
        <div className="card empty-state">{emptyMessage}</div>
      ) : (
        <div className="prompt-box">{content}</div>
      )}

      <div className="btn-row" style={{ marginTop: 16 }}>
        <button className="primary" onClick={() => void copyPrompt()} disabled={!content}>
          {t("common.copy")}
        </button>
        <button className="secondary" onClick={() => void exportTxt()} disabled={!content}>
          {t("current.exportTxt")}
        </button>
        <button className="secondary" onClick={() => void pinCurrent()} disabled={!content}>
          {t("current.pinDraft")}
        </button>
        <button className="secondary" onClick={() => void markSubmitted()} disabled={!snapshot}>
          {t("current.markSubmitted")}
        </button>
      </div>

      {message ? <div className="toast">{message}</div> : null}
    </>
  );
}
