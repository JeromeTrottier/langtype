import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./locales/en.json";

// All UI text lives in locale files (PLAN: "Ready for more native languages").
export const resources = { en: { translation: en } } as const;

void i18n.use(initReactI18next).init({
  resources,
  lng: "en",
  fallbackLng: "en",
  interpolation: { escapeValue: false },
});

export default i18n;
