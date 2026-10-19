export const locales = ["en", "es", "fr"] as const;
export type Locale = (typeof locales)[number];
export const defaultLocale: Locale = "en";

export const localeNames: Record<Locale, string> = {
  en: "English",
  es: "Español",
  fr: "Français",
};

export const storageKey = "agora.locale";

export function isLocale(value: string | null): value is Locale {
  return value === "en" || value === "es" || value === "fr";
}

/** Best-effort detection of the user's browser language at first visit. */
export function detectLocale(): Locale {
  if (typeof window === "undefined") return defaultLocale;
  const stored = window.localStorage.getItem(storageKey);
  if (isLocale(stored)) return stored;
  const preferred = window.navigator.language?.split("-")[0];
  return isLocale(preferred) ? preferred : defaultLocale;
}