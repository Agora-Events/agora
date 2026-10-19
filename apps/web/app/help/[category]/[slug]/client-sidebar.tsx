"use client";

import { useState } from "react";
import Link from "next/link";
import { ChevronDown, ChevronUp } from "@/components/ui/icons";
import type { Article } from "../../data";

interface ClientSidebarProps {
  categorySlug: string;
  currentSlug: string;
  articles: Article[];
}

export function ClientSidebar({ categorySlug, currentSlug, articles }: ClientSidebarProps) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <>
      {/* Mobile Accordion */}
      <div className="md:hidden mb-6 border-2 border-black rounded-2xl bg-white shadow-[-4px_4px_0px_0px_rgba(0,0,0,1)] overflow-hidden">
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="w-full flex items-center justify-between p-4 font-bold text-black"
        >
          <span>Related Articles</span>
          {isOpen ? <ChevronUp size={20} /> : <ChevronDown size={20} />}
        </button>
        {isOpen && (
          <ul className="border-t-2 border-black divide-y-2 divide-black">
            {articles.map((article) => {
              const isActive = article.slug === currentSlug;
              return (
                <li key={article.slug}>
                  <Link
                    href={`/help/${categorySlug}/${article.slug}`}
                    className={`block p-4 transition-colors ${
                      isActive
                        ? "bg-[#FDDA23] text-black font-semibold"
                        : "bg-white text-gray-700 hover:bg-gray-50 hover:text-black"
                    }`}
                    onClick={() => setIsOpen(false)}
                  >
                    {article.title}
                  </Link>
                </li>
              );
            })}
          </ul>
        )}
      </div>

      {/* Desktop Sidebar */}
      <aside className="hidden md:block w-64 shrink-0">
        <div className="sticky top-24 bg-white border-2 border-black rounded-2xl p-4 shadow-[-5px_5px_0px_0px_rgba(0,0,0,1)]">
          <h3 className="font-black text-lg text-black mb-4 px-2">
            Related Articles
          </h3>
          <ul className="space-y-1">
            {articles.map((article) => {
              const isActive = article.slug === currentSlug;
              return (
                <li key={article.slug}>
                  <Link
                    href={`/help/${categorySlug}/${article.slug}`}
                    className={`block px-3 py-2 rounded-xl transition-all ${
                      isActive
                        ? "bg-[#FDDA23] text-black font-bold shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)] border-2 border-black -translate-y-[1px]"
                        : "text-gray-600 hover:bg-gray-100 hover:text-black font-medium border-2 border-transparent"
                    }`}
                  >
                    {article.title}
                  </Link>
                </li>
              );
            })}
          </ul>
        </div>
      </aside>
    </>
  );
}
