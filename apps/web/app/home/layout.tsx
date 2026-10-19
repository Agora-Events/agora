import { buildMetadata } from "@/components/layout/seo";

export const metadata = buildMetadata({
  title: "Home",
  description:
    "Your personalized Agora feed — upcoming events, events you're hosting, and community picks tailored for you.",
  path: "/home",
});

export default function HomeLayout({ children }: { children: React.ReactNode }) {
  return children;
}
