"use client";

import { useTransition } from "react";
import { useTranslations } from "next-intl";
import { useLocaleContext } from "@/lib/i18n/locale-context";
import { localeNames, locales, type Locale } from "@/lib/i18n/config";

/**
 * Language switcher (Issue #1343).
 *
 * Rendered in the footer; persists the selection in `localStorage` and swaps
 * the next-intl messages immediately — no page reload required.
 */
export function LanguageSwitcher() {
  const t = useTranslations("languageSwitcher");
  const { locale, setLocale } = useLocaleContext();
  const [isPending, startTransition] = useTransition();

  const handleChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const next = event.target.value as Locale;
    if (next === locale) return;
    startTransition(() => {
      setLocale(next);
    });
  };

  return (
    <label className="flex flex-col gap-1.5 text-sm">
      <span className="text-gray-400 font-medium">{t("label")}</span>
      <select
        value={locale}
        onChange={handleChange}
        disabled={isPending}
        aria-label={t("label")}
        className="bg-white/10 text-white text-sm rounded-lg px-3 py-2 border border-white/20 focus:outline-2 focus:outline-accent transition-colors [&>option]:text-black"
      >
        {locales.map((code) => (
          <option key={code} value={code}>
            {localeNames[code]}
          </option>
        ))}
      </select>
    </label>
  );
}