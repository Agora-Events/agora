import { buildMetadata } from "@/components/layout/seo";

export const metadata = buildMetadata({
  title: "Discover Events",
  description:
    "Browse and discover the best tech, crypto, wellness, and community events happening near you and around the world.",
  path: "/discover",
});

export default function DiscoverLayout({ children }: { children: React.ReactNode }) {
  return children;
}
