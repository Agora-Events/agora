"use client";

import { useEffect, useRef, useState } from "react";
import Image from "next/image";
import Link from "next/link";

interface NotificationItem {
  id: string;
  type: "event_reminder" | "ticket_confirmation" | "organizer_update";
  title: string;
  message: string;
  time: string;
  read: boolean;
  link?: string;
}

const mockNotifications: NotificationItem[] = [
  {
    id: "1",
    type: "ticket_confirmation",
    title: "Ticket Confirmed! 🎉",
    message: "Your ticket for 'Stellar Builders Summit' has been successfully minted.",
    time: "2 hours ago",
    read: false,
    link: "/events/1",
  },
  {
    id: "2",
    type: "event_reminder",
    title: "Event starts soon",
    message: "Stellar Builders Summit is starting in 24 hours. Don't miss it!",
    time: "1 day ago",
    read: false,
    link: "/events/1",
  },
  {
    id: "3",
    type: "organizer_update",
    title: "Update from Web3 Community",
    message: "The venue for the meetup has changed. Please check the event page for details.",
    time: "3 days ago",
    read: true,
    link: "/events/2",
  },
];

function notificationIcon(type: NotificationItem["type"]) {
  switch (type) {
    case "ticket_confirmation":
      return "/icons/ticket.svg";
    case "event_reminder":
      return "/icons/calendar.svg";
    default:
      return "/icons/user-group.svg";
  }
}

/**
 * Bell-triggered notifications dropdown shown from the navbar.
 *
 * Mirrors the data shape of the full `/notifications` page but renders as a
 * compact popover so users can triage notifications without leaving the
 * current page. Issue #1547: adds a "Clear All" button in the dropdown's
 * top-right corner so users don't have to dismiss notifications one by one.
 */
export function NotificationDropdown() {
  const [isOpen, setIsOpen] = useState(false);
  const [notifications, setNotifications] = useState<NotificationItem[]>(mockNotifications);
  const containerRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);

  const unreadCount = notifications.filter((n) => !n.read).length;

  // Close on outside click.
  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isOpen]);

  // Close on Escape, return focus to the trigger.
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setIsOpen(false);
        triggerRef.current?.focus();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isOpen]);

  const clearAll = () => {
    setNotifications([]);
  };

  return (
    <div className="relative" ref={containerRef}>
      <button
        ref={triggerRef}
        type="button"
        onClick={() => setIsOpen((prev) => !prev)}
        aria-label="View notifications"
        aria-haspopup="true"
        aria-expanded={isOpen}
        className="relative w-[55.22px] h-[53px] flex items-center justify-center rounded-full border border-black bg-white
          transition-all hover:-translate-x-[2px] hover:translate-y-[2px]
          hover:shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)]
          shadow-[-4px_4px_0px_0px_rgba(0,0,0,1)]"
      >
        {unreadCount > 0 && (
          <div className="size-[9px] bg-red-500 rounded-full absolute top-[4px] right-[2px]" />
        )}
        <Image src="/icons/notification.svg" alt="Notifications" width={24} height={24} />
      </button>

      {isOpen && (
        <div
          role="menu"
          aria-label="Notifications"
          className="absolute right-0 mt-2 w-[340px] max-h-[420px] overflow-y-auto rounded-2xl border-2 border-black bg-white shadow-[-6px_6px_0_rgba(0,0,0,1)] z-50"
        >
          <div className="flex items-center justify-between gap-3 px-4 py-3 border-b-2 border-black sticky top-0 bg-white">
            <div className="flex items-center gap-2">
              <h2 className="font-bold text-ink-deep">Notifications</h2>
              {unreadCount > 0 && (
                <span className="inline-flex items-center justify-center px-2 py-0.5 rounded-full text-xs font-bold bg-[#FDDA23] text-black border-2 border-black">
                  {unreadCount}
                </span>
              )}
            </div>
            {notifications.length > 0 && (
              <button
                type="button"
                onClick={clearAll}
                aria-label="Clear all notifications"
                className="text-sm font-semibold text-error hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#FDDA23] rounded-md px-1 py-0.5 shrink-0"
              >
                Clear All
              </button>
            )}
          </div>

          {notifications.length > 0 ? (
            <ul className="flex flex-col divide-y divide-black/10">
              {notifications.map((notification) => {
                const content = (
                  <div
                    className={`flex gap-3 px-4 py-3 transition-colors hover:bg-black/5 ${
                      notification.read ? "opacity-60" : ""
                    }`}
                  >
                    <div className="w-9 h-9 shrink-0 bg-white rounded-full flex items-center justify-center border-2 border-black">
                      <Image
                        src={notificationIcon(notification.type)}
                        alt={notification.type}
                        width={18}
                        height={18}
                      />
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="font-bold text-sm text-ink-deep truncate">
                        {notification.title}
                      </p>
                      <p className="text-xs text-ink-deep/80 line-clamp-2">
                        {notification.message}
                      </p>
                      <span className="text-xs font-semibold text-gray-500">
                        {notification.time}
                      </span>
                    </div>
                  </div>
                );

                return (
                  <li key={notification.id} role="menuitem">
                    {notification.link ? (
                      <Link href={notification.link} onClick={() => setIsOpen(false)}>
                        {content}
                      </Link>
                    ) : (
                      content
                    )}
                  </li>
                );
              })}
            </ul>
          ) : (
            <p className="px-4 py-8 text-center text-sm text-ink-deep/70">
              No notifications yet.
            </p>
          )}

          <Link
            href="/notifications"
            onClick={() => setIsOpen(false)}
            className="block text-center text-sm font-bold underline hover:text-accent transition-colors px-4 py-3 border-t-2 border-black"
          >
            View all notifications
          </Link>
        </div>
      )}
    </div>
  );
}

export default NotificationDropdown;
