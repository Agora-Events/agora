"use client";

import { useEffect, useRef } from "react";
import Image from "next/image";
import { motion, AnimatePresence } from "framer-motion";

// ─── Category options ────────────────────────────────────────────────────────
const CATEGORIES = [
  { label: "Tech", icon: "/icons/Tech.svg" },
  { label: "Party", icon: "/icons/party.svg" },
  { label: "Crypto", icon: "/icons/crypto.svg" },
  { label: "Food", icon: "/icons/foods.svg" },
  { label: "Wellness", icon: "/icons/wellness.svg" },
  { label: "Gym", icon: "/icons/gym.svg" },
  { label: "Religion", icon: "/icons/religion.svg" },
];

// ─── Location options ─────────────────────────────────────────────────────────
const LOCATIONS = [
  { label: "Online", icon: "/icons/global.svg" },
  { label: "Lagos", icon: "/icons/location.svg" },
  { label: "New York", icon: "/icons/location.svg" },
  { label: "London", icon: "/icons/location.svg" },
  { label: "Singapore", icon: "/icons/location.svg" },
];

// ─── Date options ─────────────────────────────────────────────────────────────
const DATES = ["Today", "Tomorrow", "This Week", "This Month", "Any time"];

// ─── Types ────────────────────────────────────────────────────────────────────
export type FilterState = {
  date: string;
  categories: string[];
  locations: string[];
  minPrice: string;
  maxPrice: string;
};

interface FilterSidebarProps {
  isOpen: boolean;
  onClose: () => void;
  filters: FilterState;
  onFiltersChange: (filters: FilterState) => void;
}

// ─── Sidebar animations ───────────────────────────────────────────────────────
const sidebarVariants = {
  hidden: { x: "100%", opacity: 0 },
  visible: {
    x: 0,
    opacity: 1,
    transition: { type: "spring" as const, stiffness: 300, damping: 32 },
  },
  exit: {
    x: "100%",
    opacity: 0,
    transition: { duration: 0.22, ease: "easeIn" as const },
  },
};

const backdropVariants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1, transition: { duration: 0.2 } },
  exit: { opacity: 0, transition: { duration: 0.2 } },
};

// ─── Helper: toggle item in array ────────────────────────────────────────────
function toggleItem(arr: string[], item: string): string[] {
  return arr.includes(item) ? arr.filter((v) => v !== item) : [...arr, item];
}

// ─── Component ────────────────────────────────────────────────────────────────
export function FilterSidebar({
  isOpen,
  onClose,
  filters,
  onFiltersChange,
}: FilterSidebarProps) {
  const sidebarRef = useRef<HTMLDivElement>(null);

  // Close on Escape key
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    if (isOpen) document.addEventListener("keydown", handleKey);
    return () => document.removeEventListener("keydown", handleKey);
  }, [isOpen, onClose]);

  // Prevent body scroll when open
  useEffect(() => {
    document.body.style.overflow = isOpen ? "hidden" : "";
    return () => {
      document.body.style.overflow = "";
    };
  }, [isOpen]);

  // ── helpers ──
  const setDate = (date: string) => onFiltersChange({ ...filters, date });
  const toggleCategory = (cat: string) =>
    onFiltersChange({
      ...filters,
      categories: toggleItem(filters.categories, cat),
    });
  const toggleLocation = (loc: string) =>
    onFiltersChange({
      ...filters,
      locations: toggleItem(filters.locations, loc),
    });
  const setMinPrice = (minPrice: string) =>
    onFiltersChange({ ...filters, minPrice });
  const setMaxPrice = (maxPrice: string) =>
    onFiltersChange({ ...filters, maxPrice });

  const handleReset = () =>
    onFiltersChange({
      date: "",
      categories: [],
      locations: [],
      minPrice: "",
      maxPrice: "",
    });

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          {/* ── Backdrop ── */}
          <motion.div
            key="filter-backdrop"
            className="fixed inset-0 z-40 bg-black/40 backdrop-blur-[2px]"
            variants={backdropVariants}
            initial="hidden"
            animate="visible"
            exit="exit"
            onClick={onClose}
            aria-hidden="true"
          />

          {/* ── Sidebar panel ── */}
          <motion.aside
            key="filter-sidebar"
            ref={sidebarRef}
            role="dialog"
            aria-modal="true"
            aria-label="Filter events"
            className="
              fixed top-0 right-0 z-50 h-full
              w-full max-w-[360px] sm:max-w-[420px]
              bg-[#FFFBE9] shadow-[-8px_0_32px_rgba(0,0,0,0.12)]
              flex flex-col overflow-y-auto
            "
            variants={sidebarVariants}
            initial="hidden"
            animate="visible"
            exit="exit"
          >
            {/* ── Header ── */}
            <div className="flex items-center justify-between px-6 pt-7 pb-5 border-b border-black/10 shrink-0">
              <div className="flex items-center gap-2.5">
                <Image
                  src="/icons/filter.svg"
                  width={20}
                  height={20}
                  alt=""
                  aria-hidden="true"
                />
                <h2 className="font-bold text-[20px]/6 text-black">Filters</h2>
              </div>
              <div className="flex items-center gap-3">
                <button
                  onClick={handleReset}
                  className="text-[13px] font-medium text-black/50 hover:text-black transition-colors underline underline-offset-2"
                >
                  Reset all
                </button>
                <button
                  onClick={onClose}
                  aria-label="Close filters"
                  className="
                    flex items-center justify-center w-8 h-8 rounded-full
                    bg-black text-white
                    hover:bg-black/80 active:scale-95 transition-all
                  "
                >
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 14 14"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                    aria-hidden="true"
                  >
                    <path
                      d="M1 1L13 13M13 1L1 13"
                      stroke="white"
                      strokeWidth="2"
                      strokeLinecap="round"
                    />
                  </svg>
                </button>
              </div>
            </div>

            {/* ── Scrollable body ── */}
            <div className="flex flex-col gap-7 px-6 py-6 flex-1">
              {/* ─ Date section ─ */}
              <section>
                <h3 className="font-semibold text-[15px] text-black mb-3">
                  Date
                </h3>
                <div className="flex flex-wrap gap-2">
                  {DATES.map((d) => (
                    <Pill
                      key={d}
                      label={d}
                      active={filters.date === d}
                      onClick={() => setDate(filters.date === d ? "" : d)}
                    />
                  ))}
                </div>
              </section>

              <Divider />

              {/* ─ Category section ─ */}
              <section>
                <h3 className="font-semibold text-[15px] text-black mb-3">
                  Category
                </h3>
                <div className="flex flex-wrap gap-2">
                  {CATEGORIES.map((cat) => (
                    <IconPill
                      key={cat.label}
                      label={cat.label}
                      icon={cat.icon}
                      active={filters.categories.includes(cat.label)}
                      onClick={() => toggleCategory(cat.label)}
                    />
                  ))}
                </div>
              </section>

              <Divider />

              {/* ─ Location section ─ */}
              <section>
                <h3 className="font-semibold text-[15px] text-black mb-3">
                  Location
                </h3>
                <div className="flex flex-wrap gap-2">
                  {LOCATIONS.map((loc) => (
                    <IconPill
                      key={loc.label}
                      label={loc.label}
                      icon={loc.icon}
                      active={filters.locations.includes(loc.label)}
                      onClick={() => toggleLocation(loc.label)}
                    />
                  ))}
                </div>
              </section>

              <Divider />

              {/* ─ Price section ─ */}
              <section>
                <h3 className="font-semibold text-[15px] text-black mb-3">
                  Price Range
                </h3>
                <div className="flex items-center gap-3">
                  <div className="flex-1 flex flex-col gap-1">
                    <label
                      htmlFor="filter-min-price"
                      className="text-[12px] font-medium text-black/50"
                    >
                      Min ($)
                    </label>
                    <input
                      id="filter-min-price"
                      type="number"
                      min={0}
                      placeholder="0"
                      value={filters.minPrice}
                      onChange={(e) => setMinPrice(e.target.value)}
                      className="
                        h-10 px-3 rounded-xl bg-white border border-black/15
                        text-[14px] font-medium text-black
                        outline-none focus:border-black/50
                        transition-colors
                        [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none
                      "
                    />
                  </div>

                  <span className="text-black/30 mt-5 font-light text-lg">
                    —
                  </span>

                  <div className="flex-1 flex flex-col gap-1">
                    <label
                      htmlFor="filter-max-price"
                      className="text-[12px] font-medium text-black/50"
                    >
                      Max ($)
                    </label>
                    <input
                      id="filter-max-price"
                      type="number"
                      min={0}
                      placeholder="Any"
                      value={filters.maxPrice}
                      onChange={(e) => setMaxPrice(e.target.value)}
                      className="
                        h-10 px-3 rounded-xl bg-white border border-black/15
                        text-[14px] font-medium text-black
                        outline-none focus:border-black/50
                        transition-colors
                        [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none
                      "
                    />
                  </div>
                </div>
              </section>
            </div>

            {/* ── Footer CTA ── */}
            <div className="px-6 py-5 border-t border-black/10 shrink-0">
              <button
                onClick={onClose}
                className="
                  w-full h-12 rounded-[13px] bg-black text-white font-semibold text-[15px]
                  shadow-[-4px_4px_0px_0px_rgba(0,0,0,0.25)]
                  border border-black
                  hover:-translate-x-[2px] hover:translate-y-[2px]
                  hover:shadow-[-2px_2px_0px_0px_rgba(0,0,0,0.25)]
                  active:-translate-x-[4px] active:translate-y-[4px] active:shadow-none
                  transition-all
                "
              >
                Apply Filters
              </button>
            </div>
          </motion.aside>
        </>
      )}
    </AnimatePresence>
  );
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function Divider() {
  return <div className="h-px bg-black/10 -mx-6" />;
}

interface PillProps {
  label: string;
  active: boolean;
  onClick: () => void;
}

function Pill({ label, active, onClick }: PillProps) {
  return (
    <button
      onClick={onClick}
      className={`
        px-4 py-2 rounded-full text-[13px] font-medium border transition-all
        ${
          active
            ? "bg-black text-white border-black shadow-[-3px_3px_0_rgba(0,0,0,0.2)]"
            : "bg-white text-black border-black/20 hover:border-black/50"
        }
      `}
    >
      {label}
    </button>
  );
}

interface IconPillProps extends PillProps {
  icon: string;
}

function IconPill({ label, icon, active, onClick }: IconPillProps) {
  return (
    <button
      onClick={onClick}
      className={`
        flex items-center gap-1.5 px-3.5 py-2 rounded-full text-[13px] font-medium border transition-all
        ${
          active
            ? "bg-[#FDDA23] text-black border-black shadow-[-3px_3px_0_rgba(0,0,0,1)]"
            : "bg-white text-black border-black/20 hover:border-black/50"
        }
      `}
    >
      <Image src={icon} width={16} height={16} alt="" aria-hidden="true" />
      {label}
    </button>
  );
}
