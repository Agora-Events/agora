import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "My Wallet · Agora",
  description: "View your upcoming and past event tickets in one place.",
};

export default function WalletLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <>{children}</>;
}
