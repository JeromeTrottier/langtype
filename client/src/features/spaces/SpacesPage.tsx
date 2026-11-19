import { useTranslation } from "react-i18next";

export function SpacesPage() {
  const { t } = useTranslation();
  return (
    <section>
      <h1>{t("spaces.title")}</h1>
      <p>{t("spaces.empty")}</p>
    </section>
  );
}
