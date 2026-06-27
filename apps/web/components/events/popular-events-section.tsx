"use client";

import { useState, useMemo, useEffect } from "react";
import { motion, Transition } from "framer-motion";
import Image from "next/image";
import { EventCard } from "./event-card";
import { EventCardSkeleton } from "./event-card-skeleton";
import { Button } from "../ui/button";
import { EmptyState } from "../ui/empty-state";
import { FilterSidebar, FilterState } from "./filter-sidebar";
import { fetchPopularEvents, type DiscoverEvent } from "@/utils/api";

const container = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: {
      staggerChildren: 0.12,
      delayChildren: 0.15,
    },
  },
};

const item = {
  hidden: {
    opacity: 0,
    y: 16,
    filter: "blur(6px)",
  },
  show: {
    opacity: 1,
    y: 0,
    filter: "blur(0px)",
    transition: {
      duration: 0.45,
      ease: "easeOut" as Transition["ease"],
    },
  },
};

const DEFAULT_FILTERS: FilterState = {
  date: "",
  categories: [],
  locations: [],
  minPrice: "",
  maxPrice: "",
};

type PopularEventsSectionProps = {
  activeCategory?: string;
  onError: (message: string) => void;
};

export function PopularEventsSection({ activeCategory, onError }: PopularEventsSectionProps) {
  const [isFocused, setIsFocused] = useState(false);
  const [search, setSearch] = useState("");
  const [isFilterOpen, setIsFilterOpen] = useState(false);
  const [filters, setFilters] = useState<FilterState>(DEFAULT_FILTERS);
  const [events, setEvents] = useState<DiscoverEvent[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  // Pagination state
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState<number | null>(null);
  const [isLoadingMore, setIsLoadingMore] = useState(false);

  useEffect(() => {
    const controller = new AbortController();

    const loadEvents = async () => {
      try {
        setIsLoading(true);
        const data = await fetchPopularEvents(1, controller.signal);
        setEvents(data.events);
        setTotal(data.meta?.total ?? data.events.length);
        setPage(data.meta?.page ?? 1);
      } catch (err) {
        if (err instanceof Error && err.name === "AbortError") return;
        setEvents([]);
        onError("Could not load popular events");
      } finally {
        if (!controller.signal.aborted) {
          setIsLoading(false);
        }
      }
    };

    loadEvents();
    return () => controller.abort();
  }, [onError]);

  const loadMore = async () => {
    // If we already know total and have loaded all, skip
    if (total !== null && events.length >= total) return;

    const nextPage = page + 1;
    const controller = new AbortController();
    try {
      setIsLoadingMore(true);
      const data = await fetchPopularEvents(nextPage, controller.signal);

      // Append new events while avoiding duplicates
      setEvents((prev) => {
        const existingIds = new Set(prev.map((e) => e.id));
        const newEvents = data.events.filter((e) => !existingIds.has(e.id));
        return [...prev, ...newEvents];
      });

      setPage(data.meta?.page ?? nextPage);
      setTotal(data.meta?.total ?? (total ?? events.length + data.events.length));
    } catch (err) {
      if (err instanceof Error && err.name === "AbortError") return;
      onError("Could not load more events");
    } finally {
      controller.abort();
      setIsLoadingMore(false);
    }
  };

  const filteredEvents = useMemo(() => {
    let result = events;

    // 1. Search Query
    const query = search.toLowerCase().trim();
    if (query) {
      result = result.filter((event) =>
        event.title.toLowerCase().includes(query),
      );
    }

    // 2. Categories
    if (filters.categories.length > 0) {
      result = result.filter((event) =>
        filters.categories.includes(event.category),
      );
    } else if (activeCategory && activeCategory !== "All") {
      result = result.filter((event) => event.category.toLowerCase() === activeCategory.toLowerCase());
    }

    // 3. Location
    if (filters.locations.length > 0) {
      result = result.filter((event) =>
        filters.locations.some((loc) =>
          event.location.toLowerCase().includes(loc.toLowerCase()),
        ),
      );
    }

    // 4. Date
    if (filters.date && filters.date !== "Any time") {
      // Note: Since mockup dates are static strings like "Thu, 22 Jan, 1:00",
      // strict parsing for "Today", "Tomorrow" is omitted for now.
      // In a real app with timestamps, you would check the date ranges here.
    }

    // 5. Price Range
    if (filters.minPrice !== "" || filters.maxPrice !== "") {
      result = result.filter((event) => {
        const isFree = event.price.toLowerCase() === "free";
        const price = isFree ? 0 : parseFloat(event.price);

        const min = filters.minPrice !== "" ? parseFloat(filters.minPrice) : 0;
        const max =
          filters.maxPrice !== "" ? parseFloat(filters.maxPrice) : Infinity;

        return price >= min && price <= max;
      });
    }

    return result;
  }, [search, filters, events, activeCategory]);

  const widthVariants = {
    focused: { width: "12rem" },
    unfocused: { width: "8.5rem" },
  };

  const widthVariantsMobile = {
    focused: { width: "8rem", paddingLeft: "2.5rem" },
    unfocused: { width: "2.438rem" },
  };

  const allLoaded = total !== null && events.length >= total;

  return (
    <section className="px-4 bg-base py-12">
      <div className="max-w-305.25 mx-auto">
        <motion.div
          className="flex justify-between gap-3 mb-5.75"
          variants={container}
          initial="hidden"
          animate="show"
        >
          <motion.h3
            variants={item}
            className="flex items-center gap-4 font-semibold text-[15px]/16.5 sm:text-[29px]/16.5"
          >
            Popular Events
            <Image
              src="/icons/ticket.svg"
              width={24}
              height={24}
              alt="ticket icon"
            />
          </motion.h3>

          <motion.div variants={item} className="flex items-center gap-3.75">
            <div className="relative">
              <Image
                src="/icons/search.svg"
                width={24}
                height={24}
                alt="search icon"
                className="absolute left-1.75 top-1.75 pointer-events-none"
              />

              <motion.input
                className="max-sm:hidden pl-13 h-9.75 rounded-4xl bg-black pr-4 py-2 text-white outline-1 -outline-offset-1 outline-white/10 placeholder:text-white focus:outline-2 focus:-outline-offset-2 focus:outline-[#FDDA23]"
                type="text"
                placeholder="Search"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                onFocus={() => setIsFocused(true)}
                onBlur={() => setIsFocused(false)}
                variants={widthVariants}
                initial="unfocused"
                animate={isFocused ? "focused" : "unfocused"}
                transition={{ duration: 0.3, ease: "easeInOut" }}
              />

              <motion.input
                className="sm:hidden h-9.75 rounded-4xl bg-black pr-4 py-2 text-white outline-1 -outline-offset-1 outline-white/10 focus:outline-2 focus:-outline-offset-2 focus:outline-[#FDDA23]"
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                onFocus={() => setIsFocused(true)}
                onBlur={() => setIsFocused(false)}
                variants={widthVariantsMobile}
                initial="unfocused"
                animate={isFocused ? "focused" : "unfocused"}
                transition={{ duration: 0.3, ease: "easeInOut" }}
              />
            </div>

            <motion.div whileHover={{ scale: 1.05 }} whileTap={{ scale: 0.97 }}>
              <Button
                variant="dark"
                className="border-none sm:rounded-4xl! max-sm:p-0 h-9.75 sm:w-34 w-9.75"
                onClick={() => setIsFilterOpen(true)}
                aria-haspopup="dialog"
                aria-expanded={isFilterOpen}
              >
                <Image
                  src="/icons/filter.svg"
                  width={24}
                  height={24}
                  alt="filter icon"
                />
                <span className="max-sm:hidden">Filter</span>
              </Button>
            </motion.div>
          </motion.div>
        </motion.div>

        <motion.div
          className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 place-content-center "
          variants={container}
          initial="hidden"
          animate="show"
        >
          {isLoading &&
            Array.from({ length: 4 }).map((_, index) => (
              <motion.div
                key={`event-skeleton-${index}`}
                variants={item}
                className="flex"
              >
                <EventCardSkeleton />
              </motion.div>
            ))}
          {!isLoading &&
            filteredEvents.map((event) => (
            <motion.div
              key={event.id}
              variants={item}
              whileHover={{ scale: 1.02 }}
              transition={{ type: "spring", stiffness: 280, damping: 20 }}
              className="flex"
            >
              <EventCard
                id={event.id}
                title={event.title}
                date={event.date}
                location={event.location}
                price={event.price}
                imageUrl={event.imageUrl}
              />
            </motion.div>
            ))}

          {!isLoading && filteredEvents.length === 0 && (
            <div className="col-span-full">
              <EmptyState
                icon={
                  <Image
                    src="/icons/search.svg"
                    width={32}
                    height={32}
                    alt="search"
                    className="opacity-60"
                  />
                }
                title="No events found"
                description="Try adjusting your search or filters to find what you're looking for."
                action={{ label: "Clear Search", onClick: () => setSearch("") }}
              />
            </div>
          )}
        </motion.div>

        <motion.div
          className="ml-auto w-fit mt-11"
          initial={{ opacity: 0, y: 12 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.4 }}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.97 }}
        >
          {!allLoaded && (
            <Button
              variant="primary"
              className="border-none rounded-[13px]! h-11 flex items-center gap-3"
              onClick={loadMore}
            >
              {isLoadingMore ? (
                // Simple spinner using CSS
                <span className="inline-block w-4 h-4 border-2 border-white/40 border-t-white rounded-full animate-spin" />
              ) : (
                <>
                  View all Events
                  <Image
                    src="/icons/arrow-right.svg"
                    width={24}
                    height={24}
                    alt="arrow-right icon"
                  />
                </>
              )}
            </Button>
          )}

          {allLoaded && (
            <div className="text-sm text-gray-500">All events loaded</div>
          )}
        </motion.div>
      </div>

      {/* ── Filter Sidebar ── */}
      <FilterSidebar
        isOpen={isFilterOpen}
        onClose={() => setIsFilterOpen(false)}
        filters={filters}
        onFiltersChange={setFilters}
      />
    </section>
  );
}
