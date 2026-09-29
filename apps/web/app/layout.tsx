// Toast Notification System (sonner)
// Usage: import { toast } from "sonner";
//   toast.success("Event created!")
//   toast.error("Something went wrong")
//   toast.info("Loading...")
// Toaster is globally mounted below — no per-page setup needed.

import type { Metadata, Viewport } from "next";
import { Inter } from "next/font/google";
import { CookieBanner } from "@/components/layout/cookie-banner";
import { LiveAnnouncer } from "@/components/ui/live-announcer";
import { LocaleProvider } from "@/lib/i18n/locale-context";
import "./globals.css";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  metadataBase: new URL("https://agora.events"),
  title: {
    template: "Agora | %s",
    default: "Agora | Discover & Organize Events",
  },
  description:
    "Discover, organize, and register for elite Web3 and Web2 events locally and globally.",
  openGraph: {
    title: "Agora | Discover & Organize Events",
    description:
      "Discover, organize, and register for elite Web3 and Web2 events locally and globally.",
    images: [
      {
        url: "/og-image.png",
        width: 1200,
        height: 630,
        alt: "Agora Events - Discover & Organize Events",
      },
    ],
    type: "website",
  },
};

export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#FFFBE9" },
    { media: "(prefers-color-scheme: dark)", color: "#0f1115" },
  ],
};

import { Suspense } from "react";
import LoadingBar from "@/components/ui/loading-bar";
import { ThemeProvider } from "@/components/providers/theme-context";
import { AttributionCapture } from "@/components/analytics/attribution-capture";
import { TestnetBanner } from "@/components/layout/testnet-banner";

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${inter.variable} antialiased`}>
        <script
          dangerouslySetInnerHTML={{
            __html: `
              (function() {
                var theme = localStorage.getItem("theme");
                if (!theme) {
                  theme = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
                }
                document.documentElement.setAttribute("data-theme", theme);
              })();
            `,
          }}
        />
        <a className="skip-link" href="#main-content">
          Skip to main content
        </a>
        <LocaleProvider>
          <LiveAnnouncer />
          {/* Testnet warning banner — Issue #1491.
              TestnetBanner is a client component that self-gates on
              NEXT_PUBLIC_STELLAR_NETWORK so it renders nothing in production. */}
          <TestnetBanner />
          {children}
          <CookieBanner />
        </LocaleProvider>
      </body>
    </html>
  );
}
