import { useEffect, useState } from "react";
import { api } from "../api";
import { useI18n } from "../i18n";
import type { DraftRecord } from "../types";
import { formatRelativeTime, truncate } from "../utils";

export default function HistoryPage() {
  const { t } = useI18n();
  const [drafts, setDrafts] = useState<DraftRecord[]>([]);
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<DraftRecord | null>(null);
  const [message, setMessage] = useState("");

  useEffect(() => {
    void loadDrafts();
  }, []);

  async function loadDrafts(search = query) {
    const rows = search.trim()
      ? await api.searchDrafts(search, 100)
      : await api.listDrafts(100);
    setDrafts(rows);
    if (selected) {
      const updated = rows.find((d) => d.id === selected.id) ?? null;
      setSelected(updated);
    }
  }

  async function recover(id: number) {
    const draft = await api.recoverDraft(id);
    setMessage(t("history.recovered", { id: draft.id }));
  }

  async function remove(id: number) {
    await api.deleteDraft(id);
    setMessage(t("history.deleted", { id }));
    await loadDrafts();
  }

  async function clearUnpinned() {
    const count = await api.deleteAllDrafts();
    setMessage(t("history.cleared", { count }));
    setSelected(null);
    await loadDrafts();
  }

  async function togglePin(draft: DraftRecord) {
    await api.pinDraft(draft.id, !draft.pinned);
    await loadDrafts();
  }

  return (
    <>
      <header className="page-header">
        <div>
          <h1>{t("history.title")}</h1>
          <p>{t("history.subtitle")}</p>
        </div>
        <div className="btn-row">
          <button className="secondary" onClick={() => void loadDrafts()}>
            {t("common.refresh")}
          </button>
          <button className="danger" onClick={() => void clearUnpinned()}>
            {t("history.clearUnpinned")}
          </button>
        </div>
      </header>

      <div className="card" style={{ marginBottom: 16 }}>
        <input
          placeholder={t("history.searchPlaceholder")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") void loadDrafts(query);
          }}
        />
        <div className="btn-row" style={{ marginTop: 12 }}>
          <button className="primary" onClick={() => void loadDrafts(query)}>
            {t("common.search")}
          </button>
        </div>
      </div>

      <div className="grid grid-2">
        <div className="card">
          <table className="table">
            <thead>
              <tr>
                <th>{t("history.id")}</th>
                <th>{t("history.app")}</th>
                <th>{t("common.charsShort")}</th>
                <th>{t("history.updated")}</th>
              </tr>
            </thead>
            <tbody>
              {drafts.map((draft) => (
                <tr
                  key={draft.id}
                  style={{ cursor: "pointer" }}
                  onClick={() => setSelected(draft)}
                >
                  <td>
                    #{draft.id}
                    {draft.pinned ? " 📌" : ""}
                  </td>
                  <td>{draft.app_name}</td>
                  <td>{draft.char_count}</td>
                  <td>{formatRelativeTime(draft.updated_at, t)}</td>
                </tr>
              ))}
            </tbody>
          </table>
          {drafts.length === 0 ? <div className="empty-state">{t("history.empty")}</div> : null}
        </div>

        <div className="card">
          {selected ? (
            <>
              <h2>{t("history.draftTitle", { id: selected.id })}</h2>
              <p className="muted">
                {selected.app_name} · {selected.source} · {selected.char_count}{" "}
                {t("common.charsShort")}
              </p>
              <p className="preview">{truncate(selected.content, 400)}</p>
              <div className="prompt-box" style={{ minHeight: 220, marginTop: 12 }}>
                {selected.content}
              </div>
              <div className="btn-row" style={{ marginTop: 16 }}>
                <button className="primary" onClick={() => void recover(selected.id)}>
                  {t("history.copyClipboard")}
                </button>
                <button className="secondary" onClick={() => void togglePin(selected)}>
                  {selected.pinned ? t("history.unpin") : t("history.pin")}
                </button>
                <button className="danger" onClick={() => void remove(selected.id)}>
                  {t("common.delete")}
                </button>
              </div>
            </>
          ) : (
            <div className="empty-state">{t("history.selectDraft")}</div>
          )}
        </div>
      </div>

      {message ? <div className="toast">{message}</div> : null}
    </>
  );
}
