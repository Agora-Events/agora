"use client";

import Image from "next/image";
import { ReactNode } from "react";

interface CategoryPillProps {
  label: string;
  icon: string | ReactNode;
  isActive?: boolean;
  onClick?: () => void;
  backgroundColor?: string;
}

export function CategoryPill({
  label,
  icon,
  isActive = false,
  onClick,
  backgroundColor = "#DBF4C2",
}: CategoryPillProps) {
  return (
    <button
      onClick={onClick}
      style={{
        backgroundColor,
      }}
      className={`
        flex items-center gap-2 px-8 py-4 rounded-full border border-black
        font-semibold whitespace-nowrap transition-all
        shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)]
        active:translate-x-[1px] active:translate-y-[1px] active:shadow-none
        hover:opacity-90
        ${!isActive ? "opacity-50" : "opacity-100"}
      `}
    >
      {/* Render Icon */}
      {typeof icon === "string" ? (
        <Image
          src={icon}
          alt={`${label} icon`}
          width={24}
          height={24}
          className="object-contain"
        />
      ) : (
        icon
      )}

      <span className="text-black text-[16px] capitalize font-medium">
        {label}
      </span>
    </button>
  );
}