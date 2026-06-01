"use client";

import Link from "next/link";

export function NavLink({
  href,
  icon,
  text,
  isActive,
  ariaLabel,
}: {
  href: string;
  icon: string;
  text: string;
  isActive: boolean;
  ariaLabel?: string;
}) {
  return (
    <Link
      href={href}
      aria-label={ariaLabel}
      className={`flex items-center gap-1 text-[15px] font-medium transition-colors ${
        isActive ? "text-accent" : "text-black hover:opacity-80"
      }`}
    >
      <div
        className={`w-6 h-6 transition-colors ${isActive ? "bg-accent" : "bg-black"}`}
        style={{
          maskImage: `url("${icon}")`,
          WebkitMaskImage: `url("${icon}")`,
          maskRepeat: "no-repeat",
          maskPosition: "center",
          maskSize: "contain",
        }}
      />
      <span>{text}</span>
    </Link>
  );
}
