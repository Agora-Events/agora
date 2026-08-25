"use client";

import { motion, type Transition, type Variants } from "framer-motion";
import Image from "next/image";
import type { DiscoverCategory } from "@/utils/api";

type CategoryChipsProps = {
  categories: DiscoverCategory[];
  activeCategory: string;
  onCategoryChange: (category: string) => void;
  isLoading?: boolean;
};

const itemVariants: Variants = {
  hidden: { opacity: 0, y: 16, filter: "blur(4px)" },
  show: {
    opacity: 1,
    y: 0,
    filter: "blur(0px)",
    transition: {
      duration: 0.4,
      ease: "easeOut" as Transition["ease"],
    },
  },
};

export function CategoryChips({
  categories,
  activeCategory,
  onCategoryChange,
  isLoading,
}: CategoryChipsProps) {
  const allCategory = { name: "All", icon: "", color: "#FDDA23" };
  const displayCategories = [allCategory, ...categories];

  return (
    <div className="w-full overflow-x-auto flex gap-4 pb-4 snap-x snap-mandatory [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none] [scrollbar-width:none]">
      {isLoading &&
        Array.from({ length: 6 }).map((_, index) => (
          <div
            key={`skeleton-${index}`}
            className="h-[54px] min-w-[128px] animate-pulse rounded-full border-2 border-black/30 bg-black/10 snap-start shrink-0"
          />
        ))}
      {!isLoading &&
        displayCategories.map((category) => {
          const isActive = activeCategory.toLowerCase() === category.name.toLowerCase();

          return (
            <motion.div key={category.name} variants={itemVariants} className="snap-start shrink-0">
              <button
                type="button"
                onClick={() => onCategoryChange(category.name)}
                style={{
                  backgroundColor: isActive ? category.color : "transparent",
                }}
                className={`
                  flex items-center gap-2 px-[26px] py-[13px] rounded-full
                  font-medium text-[15px] whitespace-nowrap transition-all justify-center h-[54px] min-w-[100px]
                  ${
                    isActive
                      ? "border-2 border-black shadow-[-4px_4px_0px_0px_rgba(0,0,0,1)] active:translate-x-[2px] active:translate-y-[2px] active:shadow-none hover:opacity-90"
                      : "border-2 border-black/20 text-black/60 hover:border-black/50 hover:bg-black/5"
                  }
                `}
              >
                {category.icon && (
                  <Image
                    src={category.icon}
                    alt={`${category.name} icon`}
                    width={20}
                    height={20}
                    className={`mr-[2px] object-contain ${isActive ? "" : "opacity-60"}`}
                  />
                )}
                <span className={isActive ? "text-black capitalize" : "capitalize"}>
                  {category.name}
                </span>
              </button>
            </motion.div>
          );
        })}
      {!isLoading && categories.length === 0 && (
        <p className="text-sm text-black/60 shrink-0">No data available</p>
      )}
    </div>
  );
}
