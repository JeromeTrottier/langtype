import { NavLink, Route, Routes } from "react-router";
import { useTranslation } from "react-i18next";
import { SpacesPage } from "../features/spaces/SpacesPage";
import { ServerStatus } from "./ServerStatus";
import "./app.css";

function SettingsPage() {
  const { t } = useTranslation();
  return <h1>{t("settings.title")}</h1>;
}

export function App() {
  const { t } = useTranslation();
  return (
    <div className="shell">
      <header className="shell-header">
        <strong>{t("app.name")}</strong>
        <nav>
          <NavLink to="/">{t("nav.spaces")}</NavLink>
          <NavLink to="/settings">{t("nav.settings")}</NavLink>
        </nav>
      </header>
      <main className="shell-main">
        <Routes>
          <Route path="/" element={<SpacesPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Routes>
      </main>
      <footer className="shell-footer">
        <ServerStatus />
      </footer>
    </div>
  );
}
