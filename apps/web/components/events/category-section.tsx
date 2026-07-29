"use client";

import { useEffect } from "react";
import { motion, type Transition } from "framer-motion";
import useSWR from "swr";
import { fetchCategories, type DiscoverCategory } from "@/utils/api";
import { CategoryChips } from "./category-chips";

const defaultCategories: DiscoverCategory[] = [
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
  activeCategory: string;
  onCategoryChange: (category: string) => void;
  onError: (message: string) => void;
};

export function CategorySection({ activeCategory, onCategoryChange, onError }: CategorySectionProps) {
  const { data: categories, error, isLoading } = useSWR<DiscoverCategory[]>(
    "/api/events/discover/categories",
    () => fetchCategories(),
    {
      revalidateOnFocus: true,
      revalidateOnReconnect: true,
      revalidateIfStale: false,
      keepPreviousData: true,
      dedupingInterval: 2000,
    }
  );

  // Fire onError once per error occurrence — not on every render.
  useEffect(() => {
    if (error) {
      onError("Could not load categories");
    }
  }, [error, onError]);

  // While loading: show skeleton pills (pass empty array + isLoading=true).
  // After load: use API data when available, otherwise fall back to defaults.
  // If both are exhausted (error + no cache): still use defaults so the
  // section never renders as an empty gap.
  const loaded = !isLoading;
  const apiCategories = categories && categories.length > 0 ? categories : null;
  const categoriesToRender: DiscoverCategory[] = loaded
    ? (apiCategories ?? defaultCategories)
    : [];

  // Hide the entire block only when loading has finished, the fetch failed,
  // AND we somehow ended up with zero categories (shouldn't happen given the
  // defaultCategories fallback above, but guards against future regressions).
  const hasCategories = isLoading || categoriesToRender.length > 0;
  if (!hasCategories) return null;

  return (
    <section className="px-4 bg-base pt-12 pb-6">
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
          {/* Heading is suppressed while loading so it doesn't float above skeletons */}
          {!isLoading && (
            <motion.h3
              variants={item}
              className="font-semibold text-xl mb-6 flex items-center gap-2"
            >
              Browse by Category
            </motion.h3>
          )}
          {isLoading && (
            <div className="h-7 w-48 rounded-md bg-black/10 animate-pulse mb-6" />
          )}

          <CategoryChips
            categories={categoriesToRender}
            activeCategory={activeCategory}
            onCategoryChange={onCategoryChange}
            isLoading={isLoading}
          />
        </motion.div>
      </div>
    </section>
  );
}
