import type { NextConfig } from "next";
import createNextIntlPlugin from "next-intl/plugin";

const nextConfig: NextConfig = {
  turbopack: {
    root: "../../",
  },
  async headers() {
    return [
      {
        source: "/(.*)",
        headers: [
          { key: "X-Content-Type-Options", value: "nosniff" },
          { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
          { key: "X-Frame-Options", value: "SAMEORIGIN" },
          {
            key: "Permissions-Policy",
            value:
              "camera=(self), geolocation=(self), microphone=()",
          },
        ],
      },
    ];
  },
};

// Issue #1343 — internationalisation for the web app. The client-side
// LocaleProvider loads the message catalogues and switches instantly without
// URL-prefixed routing (see lib/i18n/locale-context.tsx).
const withNextIntl = createNextIntlPlugin("./lib/i18n/request.ts");

export default withNextIntl(nextConfig);