import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: {
    template: "Agora | %s",
    default: "Agora | Discover & Organize Events",
  },
  description: "Discover, organize, and register for elite Web3 and Web2 events locally and globally.",
  openGraph: {
    title: "Agora | Discover & Organize Events",
    description: "Discover, organize, and register for elite Web3 and Web2 events locally and globally.",
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

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${inter.variable} antialiased`}>{children}</body>
    </html>
  );
}
