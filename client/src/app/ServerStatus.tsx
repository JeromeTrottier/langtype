import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { api } from "../api/client";

export function ServerStatus() {
  const { t } = useTranslation();
  const health = useQuery({ queryKey: ["health"], queryFn: api.health, retry: false });
  if (health.isPending) return <p role="status">{t("server.checking")}</p>;
  if (health.isError) return <p role="alert">{t("server.down")}</p>;
  return <p role="status">{t("server.ok", { version: health.data.version })}</p>;
}
