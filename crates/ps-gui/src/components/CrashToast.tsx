import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { api } from "../api";
import { useI18n } from "../i18n";

export interface AgentCrashEvent {
  app_name: string;
  bundle_id: string;
  preview: string;
  draft_id: number | null;
}

export default function CrashToast() {
  const { t } = useI18n();
  const navigate = useNavigate();
  const [event, setEvent] = useState<AgentCrashEvent | null>(null);
  const [message, setMessage] = useState("");

  useEffect(() => {
    const unlisten = listen<AgentCrashEvent>("agent-crash", (payload) => {
      setEvent(payload.payload);
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (!event) return;
    const timer = window.setTimeout(() => setEvent(null), 5000);
    return () => window.clearTimeout(timer);
  }, [event]);

  if (!event) {
    return message ? <div className="toast">{message}</div> : null;
  }

  async function recover() {
    if (event?.draft_id) {
      await api.recoverDraft(event.draft_id);
    } else {
      await api.recoverDraft();
    }
    setMessage(t("crash.recovered"));
    setEvent(null);
  }

  return (
    <div className="crash-toast">
      <div>
        <strong>{t("crash.title", { app: event.app_name })}</strong>
        <p className="muted">{t("crash.subtitle")}</p>
        <p className="preview">{event.preview}</p>
      </div>
      <div className="btn-row">
        <button className="primary" onClick={() => void recover()}>
          {t("crash.recover")}
        </button>
        <button className="secondary" onClick={() => navigate("/history")}>
          {t("crash.openHistory")}
        </button>
        <button className="secondary" onClick={() => setEvent(null)}>
          {t("crash.dismiss")}
        </button>
      </div>
    </div>
  );
}
