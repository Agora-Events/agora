import type { Metadata } from "next";
import { buildMetadata } from "@/components/layout/seo";

export const metadata: Metadata = buildMetadata({
  title: "Help Center",
  description: "Guides and articles to help you use Agora.",
  path: "/help",
});

export default function HelpLayout({ children }: { children: React.ReactNode }) {
  return children;
}
