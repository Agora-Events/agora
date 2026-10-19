import { buildMetadata } from "@/components/layout/seo";

export const metadata = buildMetadata({
  title: "Sign In",
  description:
    "Sign in or create your Agora account to discover events, buy tickets, and connect with communities worldwide.",
  path: "/auth",
});

export default function AuthLayout({ children }: { children: React.ReactNode }) {
  return children;
}
