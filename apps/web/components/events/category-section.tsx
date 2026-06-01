"use client";

import { motion, type Transition } from "framer-motion";
import useSWR from "swr";
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

import { CategoryChips } from "./category-chips";

type CategorySectionProps = {
  activeCategory: string;
  onCategoryChange: (category: string) => void;
  onError: (message: string) => void;
};

export function CategorySection({ activeCategory, onCategoryChange, onError }: CategorySectionProps) {
  // Use SWR for category fetching with automatic caching and revalidation
  const { data: categories, error, isLoading } = useSWR<DiscoverCategory[]>(
    "/api/events/discover/categories",
    () => fetchCategories(),
    {
      // Revalidate on window focus to keep data fresh
      revalidateOnFocus: true,
      // Revalidate on reconnect
      revalidateOnReconnect: true,
      // Don't revalidate on mount if data is already cached
      revalidateIfStale: false,
      // Keep previous data while revalidating
      keepPreviousData: true,
      // Deduplicate requests within 2 seconds
      dedupingInterval: 2000,
    }
  );

  // Handle errors from SWR
  if (error && !categories) {
    onError("Could not load categories");
  }

  const categoriesToRender = categories && categories.length > 0 ? categories : defaultCategories;

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
          <motion.h3
            variants={item}
            className="font-semibold text-xl mb-6 flex items-center gap-2"
          >
            Browse by Category
          </motion.h3>

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
