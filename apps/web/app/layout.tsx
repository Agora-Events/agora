import type { Metadata } from "next";
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
        <NextIntlClientProvider locale={locale}>
          {children}
        </NextIntlClientProvider>
      </body>
    </html>
  );
}
