import { useEffect, useState } from "react";
import { api } from "../api";
import { useI18n } from "../i18n";

export default function AboutPage() {
  const { t } = useI18n();
  const [exePath, setExePath] = useState("");
  const [version] = useState("0.1.0");

  useEffect(() => {
    void api.getExecutablePath().then(setExePath);
  }, []);

  return (
    <>
      <header className="page-header">
        <div>
          <h1>{t("about.title")}</h1>
          <p>{t("about.subtitle")}</p>
        </div>
      </header>

      <div className="grid grid-2">
        <div className="card">
          <h2>Prompt Saviour</h2>
          <p className="muted">{t("about.version", { version })}</p>
          <p>{t("about.description")}</p>
          <p className="muted">{t("about.license")}</p>
        </div>
        <div className="card">
          <h2>{t("about.installPath")}</h2>
          <div className="path-box">{exePath}</div>
          <div className="btn-row" style={{ marginTop: 12 }}>
            <button className="secondary" onClick={() => void api.copyText(exePath)}>
              {t("about.copyPath")}
            </button>
          </div>
        </div>
      </div>
    </>
  );
}
