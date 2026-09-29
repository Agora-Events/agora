import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "My Wallet",
  robots: { index: false, follow: false },
};

export default function WalletLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}
