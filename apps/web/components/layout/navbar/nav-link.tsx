"use client";

import Link from "next/link";
import { motion } from "framer-motion";

export function NavLink({
  href,
  icon,
  text,
  isActive,
}: {
  href: string;
  icon: string;
  text: string;
  isActive: boolean;
}) {
  return (
    <Link
      href={href}
      className={`flex items-center gap-1 text-[15px] font-medium transition-colors ${
        isActive ? "text-[#FDDA23]" : "text-black hover:opacity-80"
      }`}
    >
      <div
        className={`w-6 h-6 transition-colors ${isActive ? "bg-[#FDDA23]" : "bg-black"}`}
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
