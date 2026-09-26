"use client";

import { useState, useMemo, useEffect, useRef } from "react";
import { motion, Transition } from "framer-motion";
import Image from "next/image";
import { EventCard } from "./event-card";
import { EventCardSkeleton } from "./event-card-skeleton";
import { EmptyState } from "../ui/empty-state";
import { Button } from "../ui/button";
import { dataEvents } from "./mockups";
import { useTranslations } from "next-intl";
import {
  FilterSidebar,
  FilterState,
  getActiveFilterCount,
} from "./filter-sidebar";

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

interface PopularEventsSectionProps {
  category: string;
  onCategoryChange: (category: string) => void;
  selectedOrganizer?: string;
  onOrganizerChange?: (organizer: string) => void;
}

type ActiveFilter = {
  key: keyof FilterState;
  value?: string;
  label: string;
};

export function PopularEventsSection({
  category,
  onCategoryChange,
  selectedOrganizer,
  onOrganizerChange,
}: PopularEventsSectionProps) {
  const t = useTranslations("discover");
  const [isFocused, setIsFocused] = useState(false);
  const [search, setSearch] = useState("");
  const debouncedSearch = useDebounce(search, SEARCH_DEBOUNCE_MS);
  const [isFilterOpen, setIsFilterOpen] = useState(false);
  const [filters, setFilters] = useState<FilterState>({
    ...DEFAULT_FILTERS,
    categories: category ? [category] : [],
  });

  const searchInputRef = useRef<HTMLInputElement>(null);
  const mobileSearchInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if modifier keys are held
      if (e.ctrlKey || e.altKey || e.metaKey || e.shiftKey) {
        return;
      }

      // 1. Pressing '/' to focus search input
      if (e.key === "/" || e.code === "Slash") {
        const target = e.target as HTMLElement | null;
        if (
          target &&
          (target.tagName === "INPUT" ||
            target.tagName === "TEXTAREA" ||
            target.isContentEditable)
        ) {
          return;
        }

        e.preventDefault();

        if (window.innerWidth < 640 && mobileSearchInputRef.current) {
          mobileSearchInputRef.current.focus();
        } else if (searchInputRef.current) {
          searchInputRef.current.focus();
        }
      }

      // 2. Pressing 'Escape' to blur and clear search field
      if (e.key === "Escape") {
        const isDesktopFocused = document.activeElement === searchInputRef.current;
        const isMobileFocused = document.activeElement === mobileSearchInputRef.current;

        if (isDesktopFocused || isMobileFocused || isFocused) {
          searchInputRef.current?.blur();
          mobileSearchInputRef.current?.blur();
          setSearch("");
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isFocused]);

  useEffect(() => {
    setFilters((currentFilters) => ({
      ...currentFilters,
      categories: category ? [category] : [],
    }));
  }, [category]);

  const handleFiltersChange = (nextFilters: FilterState) => {
    setFilters(nextFilters);
    onCategoryChange(nextFilters.categories[0] ?? "");
  };

  const activeFilters: ActiveFilter[] = [
    ...(filters.date && filters.date !== t("anyTime")
      ? [{ key: "date" as const, label: filters.date }]
      : []),
    ...filters.categories.map((category) => ({
      key: "categories" as const,
      value: category,
      label: category,
    })),
    ...filters.locations.map((location) => ({
      key: "locations" as const,
      value: location,
      label: location,
    })),
    ...(filters.minPrice
      ? [{ key: "minPrice" as const, label: `From $${filters.minPrice}` }]
      : []),
    ...(filters.maxPrice
      ? [{ key: "maxPrice" as const, label: `Up to $${filters.maxPrice}` }]
      : []),
  ];

  const removeFilter = (filter: ActiveFilter) => {
    const nextFilters = { ...filters };

    if (filter.key === "categories" || filter.key === "locations") {
      nextFilters[filter.key] = nextFilters[filter.key].filter(
        (value) => value !== filter.value,
      );
    } else {
      nextFilters[filter.key] = "";
    }

    handleFiltersChange(nextFilters);
  };

  const filteredAndSortedEvents = useMemo(() => {
    let result = events;

    // 1. Search Query
    const query = debouncedSearch.toLowerCase().trim();
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

    // 4. Price Range
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

    return sortEvents(result, sortBy);
  }, [debouncedSearch, filters, events, activeCategory, sortBy]);

  // Calculate pagination slices (at most 12 cards in the DOM)
  const totalPages = Math.ceil(filteredAndSortedEvents.length / ITEMS_PER_PAGE);
  const validCurrentPage = Math.min(Math.max(currentPage, 1), totalPages || 1);
  const startIndex = (validCurrentPage - 1) * ITEMS_PER_PAGE;
  const paginatedEvents = useMemo(() => {
    return filteredAndSortedEvents.slice(startIndex, startIndex + ITEMS_PER_PAGE);
  }, [filteredAndSortedEvents, startIndex]);

  // Handle page changes with smooth scroll respecting reduced motion
  const handlePageChange = (newPage: number) => {
    if (newPage < 1 || newPage > totalPages || newPage === validCurrentPage) return;

    setCurrentPage(newPage);

    if (gridSectionRef.current) {
      const prefersReducedMotion = window.matchMedia(
        "(prefers-reduced-motion: reduce)"
      ).matches;

      gridSectionRef.current.scrollIntoView({
        behavior: prefersReducedMotion ? "auto" : "smooth",
        block: "start",
      });
    }
  };

  // Notify parent whenever the visible count changes
  const prevCountRef = useRef<number | null>(null);
  useEffect(() => {
    if (!isLoading && onEventsChange) {
      const count = filteredAndSortedEvents.length;
      if (prevCountRef.current !== count) {
        prevCountRef.current = count;
        onEventsChange(count);
      }
    }
  }, [filteredAndSortedEvents.length, isLoading, onEventsChange]);

  const widthVariants = {
    focused: { width: "12rem" },
    unfocused: { width: "8.5rem" },
  };

  const activeFilterCount =
    (filters.date ? 1 : 0) +
    filters.categories.length +
    filters.locations.length +
    (filters.minPrice !== "" || filters.maxPrice !== "" ? 1 : 0);

  return (
    <section ref={gridSectionRef} className="px-4 bg-base py-12">
      <div className="max-w-305.25 mx-auto">
        {getActiveFilterCount(filters) > 0 && (
          <div
            className="flex flex-wrap items-center gap-2 mb-6"
            aria-label={t("activeFilters")}
          >
            {activeFilters.map((filter) => (
              <span
                key={`${filter.key}-${filter.value ?? filter.label}`}
                className="inline-flex items-center gap-1.5 rounded-full border border-black bg-white px-3 py-1.5 text-sm"
              >
                {filter.label}
                <button
                  type="button"
                  onClick={() => removeFilter(filter)}
                  aria-label={`Remove ${filter.label} filter`}
                  className="inline-flex h-5 w-5 items-center justify-center rounded-full font-bold hover:bg-black hover:text-white"
                >
                  ×
                </button>
              </span>
            ))}
            <button
              type="button"
              onClick={() => handleFiltersChange(DEFAULT_FILTERS)}
              className="text-sm font-semibold underline underline-offset-2"
            >
              Clear all
            </button>
          </div>
        )}

        <motion.div
          className="flex flex-col sm:flex-row sm:justify-between gap-3 mb-5.75"
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

          {/* ── Desktop controls ── */}
          <motion.div
            variants={item}
            className="max-sm:hidden flex items-center gap-3.75"
          >
            <div className="relative">
              <Image
                src="/icons/search.svg"
                width={24}
                height={24}
                alt="search icon"
                className="absolute left-1.75 top-1.75 pointer-events-none"
              />

              <motion.input
                ref={searchInputRef}
                className="pl-13 h-9.75 rounded-4xl bg-black pr-9 py-2 text-white outline-1 -outline-offset-1 outline-white/10 placeholder:text-white focus:outline-2 focus:-outline-offset-2 focus:outline-[#FDDA23]"
                type="text"
                placeholder="Search"
                aria-label={t("searchEvents")}
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                onFocus={() => setIsFocused(true)}
                onBlur={() => setIsFocused(false)}
                variants={widthVariants}
                initial="unfocused"
                animate={isFocused ? "focused" : "unfocused"}
                transition={{ duration: 0.3, ease: "easeInOut" }}
              />
              {!isFocused && !search && (
                <kbd
                  aria-hidden="true"
                  className="absolute right-3.5 top-1/2 -translate-y-1/2 pointer-events-none hidden sm:inline-flex h-5 min-w-5 items-center justify-center rounded border border-white/30 bg-white/10 px-1.5 text-[11px] font-mono text-white/70 shadow-xs select-none"
                >
                  /
                </kbd>
              )}
            </div>

            <div className="flex items-center gap-2">
              <label
                htmlFor="discover-sort"
                className="text-sm font-medium text-ink-soft max-sm:sr-only"
              >
                Sort by
              </label>
              <select
                id="discover-sort"
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as EventSort)}
                aria-label={t("sortEvents")}
                className="h-9.75 max-w-[10.5rem] sm:max-w-none rounded-4xl bg-black px-3 text-sm text-white outline-1 -outline-offset-1 outline-white/10 focus:outline-2 focus:-outline-offset-2 focus:outline-[#FDDA23]"
              >
                {SORT_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            <motion.div whileHover={{ scale: 1.05 }} whileTap={{ scale: 0.97 }}>
              <Button
                variant="primary"
                className="border-none rounded-4xl! h-9.75 w-34"
                onClick={() => setIsFilterOpen(true)}
                aria-haspopup="dialog"
                aria-expanded={isFilterOpen}
              >
                <Image
                  src="/icons/filter.svg"
                  width={24}
                  height={24}
                  alt=""
                  aria-hidden="true"
                />
                Filter
                {activeFilterCount > 0 && (
                  <span className="ml-1 inline-flex min-w-5 h-5 items-center justify-center rounded-full bg-black px-1.5 text-[11px] font-bold text-white">
                    {activeFilterCount}
                  </span>
                )}
              </Button>
            </motion.div>
          </motion.div>

          {/* ── Mobile controls ── */}
          <motion.div
            variants={item}
            className="sm:hidden flex items-center gap-2 w-full min-w-0"
          >
            <div className="relative flex-1 min-w-0">
              <Image
                src="/icons/search.svg"
                width={20}
                height={20}
                alt=""
                aria-hidden="true"
                className="absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none"
              />
              <input
                ref={mobileSearchInputRef}
                className="w-full pl-10 pr-9 h-9.75 rounded-4xl bg-black py-2 text-sm text-white outline-1 -outline-offset-1 outline-white/10 placeholder:text-white/70 focus:outline-2 focus:-outline-offset-2 focus:outline-[#FDDA23]"
                type="text"
                placeholder="Search events"
                aria-label={t("searchEvents")}
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
              {!search && (
                <kbd
                  aria-hidden="true"
                  className="absolute right-3.5 top-1/2 -translate-y-1/2 pointer-events-none inline-flex h-5 min-w-5 items-center justify-center rounded border border-white/30 bg-white/10 px-1.5 text-[11px] font-mono text-white/70 shadow-xs select-none"
                >
                  /
                </kbd>
              )}
            </div>

            <Button
              variant="primary"
              className="border-none rounded-4xl! h-9.75 px-4 shrink-0"
              onClick={() => setIsFilterOpen(true)}
              aria-haspopup="dialog"
              aria-expanded={isFilterOpen}
            >
              <Image
                src="/icons/filter.svg"
                width={20}
                height={20}
                alt=""
                aria-hidden="true"
              />
              Filter
              {activeFilterCount > 0 && (
                <span className="inline-flex min-w-5 h-5 items-center justify-center rounded-full bg-black px-1.5 text-[11px] font-bold text-white">
                  {activeFilterCount}
                </span>
              )}
            </Button>
          </motion.div>
        </motion.div>

        {/* ── Events Grid (Renders up to 12 cards per page) ── */}
        <motion.div
          className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 place-content-center"
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
            paginatedEvents.map((event) => (
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

          {!isLoading && filteredAndSortedEvents.length === 0 && (
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
                action={{ label: t("clearSearch"), onClick: () => setSearch("") }}
              />
            </div>
          )}
        </motion.div>

        {/* ── Pagination Controls (Hidden when totalPages <= 1) ── */}
        {!isLoading && totalPages > 1 && (
          <nav
            aria-label="Events pagination"
            className="flex items-center justify-center gap-2 mt-11"
          >
            <button
              type="button"
              disabled={validCurrentPage === 1}
              aria-disabled={validCurrentPage === 1}
              onClick={() => handlePageChange(validCurrentPage - 1)}
              className="px-3.5 h-9.5 text-sm font-medium rounded-xl border border-white/15 bg-black/40 text-white disabled:opacity-40 disabled:cursor-not-allowed hover:bg-white/10 transition-colors"
            >
              Previous
            </button>

            <div className="flex items-center gap-1.5">
              {Array.from({ length: totalPages }, (_, index) => {
                const pageNum = index + 1;
                const isCurrent = pageNum === validCurrentPage;

                return (
                  <button
                    key={`page-${pageNum}`}
                    type="button"
                    onClick={() => handlePageChange(pageNum)}
                    aria-current={isCurrent ? "page" : undefined}
                    className={`min-w-9.5 h-9.5 px-3 text-sm font-medium rounded-xl transition-colors ${
                      isCurrent
                        ? "bg-[#FDDA23] text-black font-semibold"
                        : "bg-black/40 text-white border border-white/15 hover:bg-white/10"
                    }`}
                  >
                    {pageNum}
                  </button>
                );
              })}
            </div>

            <button
              type="button"
              disabled={validCurrentPage === totalPages}
              aria-disabled={validCurrentPage === totalPages}
              onClick={() => handlePageChange(validCurrentPage + 1)}
              className="px-3.5 h-9.5 text-sm font-medium rounded-xl border border-white/15 bg-black/40 text-white disabled:opacity-40 disabled:cursor-not-allowed hover:bg-white/10 transition-colors"
            >
              Next
            </button>
          </nav>
        )}
      </div>

      {/* ── Filter Sidebar ── */}
      <FilterSidebar
        isOpen={isFilterOpen}
        onClose={() => setIsFilterOpen(false)}
        filters={filters}
        onFiltersChange={handleFiltersChange}
      />
    </section>
  );
}