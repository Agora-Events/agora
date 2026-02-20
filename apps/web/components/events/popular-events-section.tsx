"use client";

import { useState, useMemo } from "react";
import { motion, Transition } from "framer-motion";
import Image from "next/image";
import { EventCard } from "./event-card";
import { Button } from "../ui/button";
import { dataEvents } from "./mockups";
import { FilterSidebar, FilterState } from "./filter-sidebar";

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

export function PopularEventsSection() {
  const [isFocused, setIsFocused] = useState(false);
  const [search, setSearch] = useState("");
  const [isFilterOpen, setIsFilterOpen] = useState(false);
  const [filters, setFilters] = useState<FilterState>(DEFAULT_FILTERS);

  const filteredEvents = useMemo(() => {
    const query = search.toLowerCase().trim();
    if (!query) return dataEvents;

    return dataEvents.filter((event) =>
      event.title.toLowerCase().includes(query),
    );
  }, [search]);

  const widthVariants = {
    focused: { width: "12rem" },
    unfocused: { width: "8.5rem" },
  };

  const widthVariantsMobile = {
    focused: { width: "8rem", paddingLeft: "2.5rem" },
    unfocused: { width: "2.438rem" },
  };

  return (
    <section className="px-4 bg-[#FFFBE9] py-12">
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
                backgroundColor="bg-black"
                shadowColor="transparent"
                textColor="text-white"
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
          className="grid min-[900px]:grid-cols-2 gap-8 place-content-center "
          variants={container}
          initial="hidden"
          animate="show"
        >
          {filteredEvents.map((event) => (
            <motion.div
              key={event.id}
              variants={item}
              whileHover={{ scale: 1.02 }}
              transition={{ type: "spring", stiffness: 280, damping: 20 }}
              className="flex"
            >
              <EventCard
                title={event.title}
                date={event.date}
                location={event.location}
                price={event.price}
                imageUrl={event.imageUrl}
              />
            </motion.div>
          ))}

          {filteredEvents.length === 0 && (
            <motion.p
              variants={item}
              className="col-span-full text-center text-black "
            >
              No events found
            </motion.p>
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
          <Button
            backgroundColor="bg-[#FDDA23]"
            shadowColor="transparent"
            className="border-none rounded-[13px]! h-11"
          >
            View all Events
            <Image
              src="/icons/arrow-right.svg"
              width={24}
              height={24}
              alt="arrow-right icon"
            />
          </Button>
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
