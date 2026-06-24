import { NavLink, Route, Routes } from "react-router-dom";
import CrashToast from "./components/CrashToast";
import { useI18n } from "./i18n";
import AboutPage from "./pages/About";
import CurrentPromptPage from "./pages/CurrentPrompt";
import DashboardPage from "./pages/Dashboard";
import HistoryPage from "./pages/History";
import PermissionsPage from "./pages/Permissions";
import SettingsPage from "./pages/Settings";

export default function App() {
  const { t } = useI18n();

  const links = [
    { to: "/", label: t("nav.dashboard") },
    { to: "/current", label: t("nav.current") },
    { to: "/history", label: t("nav.history") },
    { to: "/permissions", label: t("nav.permissions") },
    { to: "/settings", label: t("nav.settings") },
    { to: "/about", label: t("nav.about") },
  ];

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <strong>Prompt Saviour</strong>
          <span>{t("brand.subtitle")}</span>
        </div>
        <nav className="nav">
          {links.map((link) => (
            <NavLink
              key={link.to}
              to={link.to}
              end={link.to === "/"}
              className={({ isActive }) => (isActive ? "active" : undefined)}
            >
              {link.label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main className="main">
        <Routes>
          <Route path="/" element={<DashboardPage />} />
          <Route path="/current" element={<CurrentPromptPage />} />
          <Route path="/history" element={<HistoryPage />} />
          <Route path="/permissions" element={<PermissionsPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/about" element={<AboutPage />} />
        </Routes>
      </main>
      <CrashToast />
    </div>
  );
}
