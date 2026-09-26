import { getRequestConfig } from "next-intl/server";
import { defaultLocale, isLocale } from "./config";

/**
 * Request-scoped locale resolution for next-intl's server APIs.
 *
 * The app uses client-side language switching persisted in `localStorage`, so
 * server rendering always falls back to the default locale. Client components
 * re-render instantly when the user switches languages (see locale-context.tsx).
 */
export default getRequestConfig(async ({ requestLocale }) => {
  const requested = await requestLocale;
  const locale = isLocale(requested) ? requested : defaultLocale;
  return {
    locale,
    messages: (await import(`../../messages/${locale}.json`)).default,
  };
});