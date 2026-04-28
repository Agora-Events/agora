"use client";

import { motion, type Transition } from "framer-motion";
import Image from "next/image";
import { useEffect, useState } from "react";
import { fetchCategories, type DiscoverCategory } from "@/utils/api";

const defaultCategories = [
  { name: "Tech", icon: "/icons/Tech.svg", color: "#DBF4B9" },
  { name: "Party", icon: "/icons/party.svg", color: "#FFA4D5" },
  { name: "global", icon: "/icons/global.svg", color: "#B9C7FE" },
  { name: "Art & Craft", icon: "/icons/brush.svg", color: "#DEC6FA" },
  { name: "Religion", icon: "/icons/religion.svg", color: "#AAC8FA" },
  { name: "Gym", icon: "/icons/gym.svg", color: "#FFF9CA" },
  { name: "Crypto", icon: "/icons/crypto.svg", color: "#FFC4C7" },
  { name: "Wellness", icon: "/icons/wellness.svg", color: "#C2FE8B" },
  { name: "Foods", icon: "/icons/foods.svg", color: "#FFBEBE" },
  { name: "AI", icon: "/icons/ai.svg", color: "#FC94FC" },
];

const container = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

const item = {
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

type CategorySectionProps = {
  onError: (message: string) => void;
};

export function CategorySection({ onError }: CategorySectionProps) {
  const [categories, setCategories] = useState<DiscoverCategory[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    let active = true;

    const loadCategories = async () => {
      try {
        const data = await fetchCategories();
        if (active) {
          setCategories(data);
        }
      } catch {
        if (active) {
          setCategories([]);
          onError("Could not load categories");
        }
      } finally {
        if (active) {
          setIsLoading(false);
        }
      }
    };

    loadCategories();
    return () => {
      active = false;
    };
  }, [onError]);

  const categoriesToRender = categories.length > 0 ? categories : defaultCategories;

  return (
    <section className="px-4 bg-[#FFFBE9] pt-12 pb-6">
      <div className="mx-auto max-w-[1221px]">
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          className="mb-10 max-w-2xl"
        >
          <h1 className="text-4xl sm:text-5xl font-bold mb-4 italic">
            Discover Events
          </h1>
          <p className="text-gray-600 text-sm sm:text-base leading-relaxed">
            Explore popular events near you, browse by category, or check out
            some of the great community calendars.
          </p>
        </motion.div>

        <motion.div variants={container} initial="hidden" animate="show">
          <motion.h3
            variants={item}
            className="font-semibold text-xl mb-6 flex items-center gap-2"
          >
            Browse by Category
          </motion.h3>

          <motion.div variants={container} className="flex flex-wrap gap-4">
            {isLoading &&
              Array.from({ length: 6 }).map((_, index) => (
                <div
                  key={`skeleton-${index}`}
                  className="h-12 w-36 animate-pulse rounded-full border-2 border-black/30 bg-black/10"
                />
              ))}
            {!isLoading &&
              categoriesToRender.map((category) => (
              <motion.div key={category.name} variants={item}>
                <button
                  style={{ backgroundColor: category.color }}
                  className={`
                    flex items-center gap-2 px-[26px] py-[13px] rounded-full border-2 border-black
                    font-medium text-[15px] whitespace-nowrap transition-all
                    shadow-[-4px_4px_0px_0px_rgba(0,0,0,1)]
                    active:translate-x-[2px] active:translate-y-[2px] active:shadow-none
                    hover:opacity-90 min-w-32 justify-center
                  `}
                >
                  <Image
                    src={category.icon}
                    alt={`${category.name} icon`}
                    width={20}
                    height={20}
                    className="mr-[2px] object-contain"
                  />
                  <span className="text-black capitalize">{category.name}</span>
                </button>
              </motion.div>
              ))}
            {!isLoading && categories.length === 0 && (
              <p className="text-sm text-black/60">No data available</p>
            )}
          </motion.div>
        </motion.div>
      </div>
    </section>
  );
}
