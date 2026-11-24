// Toast Notification System (sonner)
// Usage: import { toast } from "sonner";
//   toast.success("Event created!")
//   toast.error("Something went wrong")
//   toast.info("Loading...")
// Toaster is globally mounted below — no per-page setup needed.

import type { Metadata, Viewport } from "next";
import { Inter } from "next/font/google";
import { cookies } from "next/headers";
import { NextIntlClientProvider } from "next-intl";
import "./globals.css";

const SUPPORTED_LOCALES = ["en", "es", "fr"] as const;
type Locale = (typeof SUPPORTED_LOCALES)[number];

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

import { Suspense } from "react";
import LoadingBar from "@/components/ui/loading-bar";
import { ThemeProvider } from "@/components/providers/theme-context";
import { AttributionCapture } from "@/components/analytics/attribution-capture";
import { TestnetBanner } from "@/components/layout/testnet-banner";

export default function RootLayout({
export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const cookieStore = await cookies();
  const savedLocale = cookieStore.get("NEXT_LOCALE")?.value;
  const locale: Locale =
    SUPPORTED_LOCALES.find((supportedLocale) => supportedLocale === savedLocale) ??
    "en";

  return (
    <html lang={locale}>
      <body className={`${inter.variable} antialiased`}>
        <a className="skip-link" href="#main-content">
          Skip to main content
        </a>
        <LocaleProvider>
          <LiveAnnouncer />
          {/* Testnet warning banner — Issue #1491.
              TestnetBanner is a client component that self-gates on
              NEXT_PUBLIC_STELLAR_NETWORK so it renders nothing in production. */}
          <TestnetBanner />
        <NextIntlClientProvider locale={locale}>
          {children}
        </NextIntlClientProvider>
      </body>
    </html>
  );
}
