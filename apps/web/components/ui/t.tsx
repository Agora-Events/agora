"use client";

import { useTranslations } from "next-intl";

/**
 * Translatable text helper for use inside server-rendered pages.
 *
 * Server components cannot consume `useTranslations` directly; mounting this
 * client component lets a single string re-translate instantly when the locale
 * changes without converting the whole page to a client component.
 */
export function T({
  ns,
  k,
  values,
}: {
  ns: Parameters<typeof useTranslations>[0] & string;
  k: string;
  values?: Record<string, string | number>;
}) {
  const t = useTranslations(ns);
  return <>{t(k as never, values as never)}</>;
}