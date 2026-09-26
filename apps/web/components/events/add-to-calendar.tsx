"use client";

import React, { useState, useRef, useEffect } from "react";
import Image from "next/image";
import {
  CalendarEventInput,
  buildIcsFile,
  buildGoogleCalendarUrl,
} from "@/utils/calendar";

interface AddToCalendarProps {
  event: CalendarEventInput;
  className?: string;
}

export function AddToCalendar({ event, className = "" }: AddToCalendarProps) {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(e.target as Node)
      ) {
        setIsOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const handleDownloadIcs = () => {
    const icsContent = buildIcsFile(event);
    const blob = new Blob([icsContent], {
      type: "text/calendar;charset=utf-8",
    });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    const sanitizedTitle = (event.title || "event")
      .toLowerCase()
      .replace(/[^a-z0-9]/g, "-")
      .replace(/-+/g, "-");
    link.setAttribute("download", `${sanitizedTitle}.ics`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
    setIsOpen(false);
  };

  const googleCalendarUrl = buildGoogleCalendarUrl(event);

  return (
    <div className={`relative inline-block ${className}`} ref={dropdownRef}>
      <button
        type="button"
        onClick={() => setIsOpen(!isOpen)}
        aria-expanded={isOpen}
        aria-haspopup="true"
        className="flex items-center gap-2.5 px-4 py-2.5 rounded-full border border-black bg-white dark:bg-surface text-black dark:text-white font-semibold text-sm transition-all hover:-translate-x-[2px] hover:translate-y-[2px] hover:shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)] active:translate-x-0 active:translate-y-0 active:shadow-none shadow-[-4px_4px_0px_0px_rgba(0,0,0,1)]"
      >
        <Image
          src="/icons/notification.svg"
          width={18}
          height={18}
          alt="Calendar"
          className="dark:invert"
        />
        <span>Add to Calendar</span>
        <svg
          className={`w-4 h-4 transition-transform duration-200 ${
            isOpen ? "rotate-180" : ""
          }`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>

      {isOpen && (
        <div className="absolute left-0 sm:left-auto sm:right-0 mt-2 w-56 rounded-2xl border-2 border-black bg-white dark:bg-surface shadow-[4px_4px_0px_0px_#000] z-50 overflow-hidden py-1">
          <a
            href={googleCalendarUrl}
            target="_blank"
            rel="noopener noreferrer"
            onClick={() => setIsOpen(false)}
            className="flex items-center gap-3 px-4 py-3 text-sm font-medium text-black dark:text-white hover:bg-cream dark:hover:bg-surface-alt transition-colors border-b border-gray-100 dark:border-gray-800"
          >
            <svg
              className="w-4 h-4 text-blue-600 shrink-0"
              viewBox="0 0 24 24"
              fill="currentColor"
            >
              <path d="M19 4h-1V2h-2v2H8V2H6v2H5c-1.11 0-1.99.9-1.99 2L3 20c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 16H5V10h14v10zm0-12H5V6h14v2z" />
            </svg>
            <span>Google Calendar</span>
          </a>

          <button
            type="button"
            onClick={handleDownloadIcs}
            className="w-full flex items-center gap-3 px-4 py-3 text-sm font-medium text-black dark:text-white hover:bg-cream dark:hover:bg-surface-alt transition-colors text-left"
          >
            <svg
              className="w-4 h-4 text-emerald-600 shrink-0"
              viewBox="0 0 24 24"
              fill="currentColor"
            >
              <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z" />
            </svg>
            <span>Apple / Outlook (.ics)</span>
          </button>
        </div>
      )}
    </div>
  );
}
