"use client";

import Link from "next/link";
import { motion } from "framer-motion";

export function MobileNavLink({
  href,
  icon,
  text,
  i,
  isActive,
}: {
  href: string;
  icon: string;
  text: string;
  i: number;
  isActive: boolean;
}) {
  const linkVariants = {
    closed: { opacity: 0, x: 20 },
    open: (i: number) => ({
      opacity: 1,
      x: 0,
      transition: {
        delay: i * 0.1,
        duration: 0.4,
        ease: "easeOut" as const,
      },
    }),
  };

  return (
    <motion.div custom={i} variants={linkVariants}>
      <Link
        href={href}
        className={`flex items-center gap-3 text-lg font-medium transition-colors p-2 rounded-lg ${
          isActive ? "text-[#FDDA23]" : "hover:opacity-80"
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
    </motion.div>
  );
}
