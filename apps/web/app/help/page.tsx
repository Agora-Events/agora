"use client";

import { useState } from "react";
import Image from "next/image";
import Link from "next/link";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";

// Icon imports — all sourced from /public/icons/
import SearchIcon from "@/public/icons/search.svg";
import HelpCircleIcon from "@/public/icons/help-circle.svg";
import TicketIcon from "@/public/icons/ticket.svg";
import AddCircleIcon from "@/public/icons/add-circle.svg";
import DollarCircleIcon from "@/public/icons/dollar-circle.svg";
import CryptoIcon from "@/public/icons/crypto.svg";
import BubbleChatIcon from "@/public/icons/bubble-chat.svg";
import NotificationIcon from "@/public/icons/notification.svg";
import MegaphoneIcon from "@/public/icons/megaphone.svg";
import UserGroupIcon from "@/public/icons/user-group.svg";

interface TopicCategory {
  title: string;
  /** slug used for /help/[category] routing */
  slug: string;
  icon: string;
  articleCount: number;
  description: string;
}

const categories: TopicCategory[] = [
  {
    title: "Getting Started",
    slug: "getting-started",
    icon: HelpCircleIcon,
    articleCount: 8,
    description: "New to Agora? Start here.",
  },
  {
    title: "Buying Tickets",
    slug: "buying-tickets",
    icon: TicketIcon,
    articleCount: 5,
    description: "Find, purchase, and manage tickets.",
  },
  {
    title: "Creating Events",
    slug: "creating-events",
    icon: AddCircleIcon,
    articleCount: 11,
    description: "Publish and manage your events.",
  },
  {
    title: "Payments",
    slug: "payments",
    icon: DollarCircleIcon,
    articleCount: 6,
    description: "Billing, refunds, and payouts.",
  },
  {
    title: "Stellar & Web3",
    slug: "stellar-web3",
    icon: CryptoIcon,
    articleCount: 9,
    description: "Wallets, XLM, and on-chain features.",
  },
  {
    title: "Notifications",
    slug: "notifications",
    icon: NotificationIcon,
    articleCount: 4,
    description: "Manage alerts and preferences.",
  },
  {
    title: "Organizer Tools",
    slug: "organizer-tools",
    icon: MegaphoneIcon,
    articleCount: 7,
    description: "Analytics, promotion, and check-in.",
  },
  {
    title: "Community & Chat",
    slug: "community-chat",
    icon: BubbleChatIcon,
    articleCount: 5,
    description: "Messaging and attendee interaction.",
  },
  {
    title: "Account & Profile",
    slug: "account-profile",
    icon: UserGroupIcon,
    articleCount: 6,
    description: "Settings, privacy, and security.",
  },
];

export default function HelpCenterPage() {
  const [query, setQuery] = useState("");

  const filtered = query.trim()
    ? categories.filter(
        (c) =>
          c.title.toLowerCase().includes(query.toLowerCase()) ||
          c.description.toLowerCase().includes(query.toLowerCase())
      )
    : categories;

  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />

      {/* ── Hero ─────────────────────────────────────────────────── */}
      <section className="mt-20 px-4 py-16 md:py-24 flex flex-col items-center text-center">
        {/* Badge */}
        <span className="inline-flex items-center gap-1.5 px-4 py-1.5 rounded-full border border-black bg-[#FDDA23] text-xs font-semibold tracking-wide mb-6 shadow-[-3px_3px_0px_0px_rgba(0,0,0,1)]">
          <Image src={HelpCircleIcon} alt="" width={14} height={14} />
          Help Center
        </span>

        <h1 className="text-4xl md:text-5xl lg:text-6xl font-black tracking-tight text-black leading-tight max-w-2xl">
          How can we{" "}
          <span className="relative inline-block">
            help you?
            {/* yellow underline accent */}
            <span
              aria-hidden
              className="absolute left-0 -bottom-1 w-full h-3 bg-[#FDDA23] -z-10 rounded-sm"
            />
          </span>
        </h1>

        <p className="mt-5 text-base md:text-lg text-gray-600 max-w-md">
          Browse our help topics or search for a specific question below.
        </p>

        {/* Search bar */}
        <div className="mt-8 w-full max-w-xl relative">
          <div className="flex items-center gap-3 bg-white border-2 border-black rounded-full px-5 py-3 shadow-[-5px_5px_0px_0px_rgba(0,0,0,1)] focus-within:shadow-[-3px_3px_0px_0px_rgba(0,0,0,1)] focus-within:-translate-x-[1px] focus-within:translate-y-[1px] transition-all">
            <Image
              src={SearchIcon}
              alt="Search"
              width={20}
              height={20}
              className="shrink-0 opacity-50"
            />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search for articles, topics…"
              className="flex-1 bg-transparent text-sm font-medium text-black placeholder:text-gray-400 outline-none"
              aria-label="Search help articles"
            />
            {query && (
              <button
                onClick={() => setQuery("")}
                className="text-gray-400 hover:text-black text-lg leading-none transition-colors"
                aria-label="Clear search"
              >
                ×
              </button>
            )}
          </div>
        </div>

        {/* Quick-link pills */}
        <div className="mt-5 flex flex-wrap justify-center gap-2">
          {["Getting Started", "Payments", "Stellar & Web3"].map((label) => (
            <button
              key={label}
              onClick={() => setQuery(label)}
              className="px-3 py-1 rounded-full border border-black text-xs font-medium bg-white hover:bg-[#FDDA23] transition-colors"
            >
              {label}
            </button>
          ))}
        </div>
      </section>

      {/* ── Topic Grid ───────────────────────────────────────────── */}
      <section className="px-4 md:px-8 pb-20 max-w-5xl mx-auto w-full">
        {/* Section heading */}
        <div className="flex items-center justify-between mb-8">
          <h2 className="text-xl font-bold text-black">
            {query ? `Results for "${query}"` : "Browse Topics"}
          </h2>
          {query && (
            <span className="text-sm text-gray-500">
              {filtered.length} topic{filtered.length !== 1 ? "s" : ""}
            </span>
          )}
        </div>

        {filtered.length === 0 ? (
          /* Empty state */
          <div className="flex flex-col items-center py-20 text-center">
            <span className="text-5xl mb-4">🔍</span>
            <p className="font-semibold text-black">No topics found</p>
            <p className="text-sm text-gray-500 mt-1">
              Try a different search term, or{" "}
              <button
                onClick={() => setQuery("")}
                className="underline text-black"
              >
                browse all topics
              </button>
              .
            </p>
          </div>
        ) : (
          /* 1-col mobile → 2-col tablet → 3-col desktop */
          <ul className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {filtered.map((category) => (
              <li key={category.slug}>
                <Link
                  href={`/help/${category.slug}`}
                  className="group flex flex-col h-full bg-white border-2 border-black rounded-2xl p-6
                             shadow-[-5px_5px_0px_0px_rgba(0,0,0,1)]
                             hover:shadow-[-3px_3px_0px_0px_rgba(0,0,0,1)]
                             hover:-translate-x-[1px] hover:translate-y-[1px]
                             active:shadow-none active:-translate-x-[3px] active:translate-y-[3px]
                             transition-all duration-150"
                >
                  {/* Icon badge */}
                  <div className="w-12 h-12 rounded-xl border-2 border-black bg-[#FDDA23] flex items-center justify-center mb-4 shadow-[-3px_3px_0px_0px_rgba(0,0,0,1)] group-hover:shadow-[-1px_1px_0px_0px_rgba(0,0,0,1)] group-hover:-translate-x-[1px] group-hover:translate-y-[1px] transition-all">
                    <Image
                      src={category.icon}
                      alt={category.title}
                      width={22}
                      height={22}
                    />
                  </div>

                  {/* Text */}
                  <h3 className="font-bold text-base text-black leading-snug mb-1">
                    {category.title}
                  </h3>
                  <p className="text-sm text-gray-500 flex-1 leading-relaxed">
                    {category.description}
                  </p>

                  {/* Footer row */}
                  <div className="mt-4 pt-4 border-t border-gray-100 flex items-center justify-between">
                    <span className="inline-flex items-center gap-1 text-xs font-semibold text-gray-500">
  {category.articleCount} article
  {category.articleCount !== 1 ? "s" : ""}
</span>

                    {/* Arrow */}
                    <span className="text-black opacity-0 group-hover:opacity-100 transition-opacity text-sm font-bold">
                      →
                    </span>
                  </div>
                </Link>
              </li>
            ))}
          </ul>
        )}
      </section>

      {/* ── Contact Banner ───────────────────────────────────────── */}
      <section className="px-4 md:px-8 pb-24 max-w-5xl mx-auto w-full">
        <div className="border-2 border-black rounded-2xl bg-[#FDDA23] p-8 shadow-[-6px_6px_0px_0px_rgba(0,0,0,1)] flex flex-col sm:flex-row items-start sm:items-center justify-between gap-6">
          <div>
            <p className="font-black text-lg text-black">
              Still can&apos;t find what you need?
            </p>
            <p className="text-sm text-black/70 mt-1">
              Our support team is happy to help you out.
            </p>
          </div>
          <Link
            href="/contact"
            className="shrink-0 inline-flex items-center gap-2 px-6 py-3 rounded-full border-2 border-black bg-black text-white text-sm font-semibold
                       shadow-[-4px_4px_0px_0px_rgba(253,218,35,1)]
                       hover:shadow-[-2px_2px_0px_0px_rgba(253,218,35,1)]
                       hover:-translate-x-[1px] hover:translate-y-[1px]
                       active:shadow-none active:-translate-x-[3px] active:translate-y-[3px]
                       transition-all"
          >
            Contact Support →
          </Link>
        </div>
      </section>

      <Footer />
    </main>
  );
}
