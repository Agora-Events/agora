"use client";

import { useCallback, useEffect, useState } from "react";
import Image from "next/image";
import { useRouter } from "next/navigation";

type NotificationItem = {
  id: string;
  title: string;
  message: string;
  createdAt: string;
  read: boolean;
  href?: string;
};

function parseNotifications(value: unknown): NotificationItem[] {
  if (
    typeof value !== "object" ||
    value === null ||
    !("notifications" in value) ||
    !Array.isArray(value.notifications)
  ) {
    throw new Error("Notifications response has an invalid format.");
  }

  return value.notifications.map((item: unknown) => {
    if (
      typeof item !== "object" ||
      item === null ||
      !("id" in item) ||
      typeof item.id !== "string" ||
      !("title" in item) ||
      typeof item.title !== "string" ||
      !("message" in item) ||
      typeof item.message !== "string" ||
      !("createdAt" in item) ||
      typeof item.createdAt !== "string" ||
      !("read" in item) ||
      typeof item.read !== "boolean"
    ) {
      throw new Error("A notification has an invalid format.");
    }

    const href = "href" in item ? item.href : undefined;
    if (href !== undefined && typeof href !== "string") {
      throw new Error("A notification has an invalid link.");
    }

    return {
      id: item.id,
      title: item.title,
      message: item.message,
      createdAt: item.createdAt,
      read: item.read,
      href,
    };
  });
}

export function NotificationDropdown() {
  const router = useRouter();
  const [notifications, setNotifications] = useState<NotificationItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isOpen, setIsOpen] = useState(false);
  const [error, setError] = useState("");
  const [markingId, setMarkingId] = useState<string | null>(null);
  const unreadCount = notifications.filter((notification) => !notification.read).length;

  const refreshNotifications = useCallback(async (signal?: AbortSignal) => {
    try {
      const response = await fetch("/api/v1/notifications", {
        credentials: "same-origin",
        cache: "no-store",
        signal,
      });
      if (!response.ok) {
        throw new Error(`Notifications request failed (${response.status}).`);
      }

      const data: unknown = await response.json();
      const parsed = parseNotifications(data);
      setNotifications(parsed);
      setError("");
    } catch (requestError) {
      if (requestError instanceof DOMException && requestError.name === "AbortError") {
        return;
      }
      console.error("Unable to load notifications:", requestError);
      setError("Notifications could not be loaded.");
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    const controller = new AbortController();
    void refreshNotifications(controller.signal);

    const intervalId = window.setInterval(() => {
      void refreshNotifications();
    }, 30_000);
    const handleFocus = () => void refreshNotifications();
    window.addEventListener("focus", handleFocus);

    return () => {
      controller.abort();
      window.clearInterval(intervalId);
      window.removeEventListener("focus", handleFocus);
    };
  }, [refreshNotifications]);

  const markAsRead = async (notification: NotificationItem) => {
    if (markingId) {
      return;
    }

    if (!notification.read) {
      setMarkingId(notification.id);
      setError("");
      try {
        const response = await fetch(
          `/api/v1/notifications/${encodeURIComponent(notification.id)}/read`,
          {
            method: "PATCH",
            credentials: "same-origin",
          },
        );
        if (!response.ok) {
          throw new Error(`Mark notification as read failed (${response.status}).`);
        }

        setNotifications((current) =>
          current.map((item) =>
            item.id === notification.id ? { ...item, read: true } : item,
          ),
        );
      } catch (requestError) {
        console.error("Unable to mark notification as read:", requestError);
        setError("This notification could not be marked as read.");
        return;
      } finally {
        setMarkingId(null);
      }
    }

    setIsOpen(false);
    if (notification.href?.startsWith("/") && !notification.href.startsWith("//")) {
      router.push(notification.href);
    }
  };

  return (
    <div className="relative">
      <button
        type="button"
        aria-label={
          unreadCount > 0
            ? `Notifications, ${unreadCount} unread`
            : "Notifications"
        }
        aria-expanded={isOpen}
        onClick={() => setIsOpen((open) => !open)}
        className="relative flex h-[53px] w-[55px] items-center justify-center rounded-full border border-black bg-white px-[10px] py-[10px] shadow-[-4px_4px_0px_0px_rgba(0,0,0,1)] transition hover:-translate-x-[2px] hover:translate-y-[2px] hover:shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)]"
      >
        <Image
          src="/icons/notification.svg"
          alt=""
          width={24}
          height={24}
        />
        {unreadCount > 0 && (
          <span className="absolute -right-2 -top-2 flex h-5 min-w-5 items-center justify-center rounded-full bg-red-600 px-1 text-[11px] font-semibold text-white">
            {unreadCount > 99 ? "99+" : unreadCount}
          </span>
        )}
      </button>

      {isOpen && (
        <section
          aria-label="Notifications"
          className="absolute right-0 top-[calc(100%+12px)] z-[60] w-[min(360px,calc(100vw-2rem))] overflow-hidden rounded-2xl border border-[#F0EAD6] bg-white shadow-lg"
        >
          <div className="flex items-center justify-between border-b border-[#F0EAD6] px-4 py-3">
            <h2 className="font-semibold text-[#1A1A1A]">Notifications</h2>
            {unreadCount > 0 && (
              <span className="text-xs text-gray-500">{unreadCount} unread</span>
            )}
          </div>

          {error && (
            <p role="alert" className="px-4 py-3 text-sm text-red-600">
              {error}
            </p>
          )}

          {isLoading && notifications.length === 0 ? (
            <p className="px-4 py-6 text-center text-sm text-gray-500">
              Loading notifications...
            </p>
          ) : notifications.length === 0 ? (
            <p className="px-4 py-6 text-center text-sm text-gray-500">
              No notifications yet.
            </p>
          ) : (
            <ul className="max-h-[min(420px,70vh)] overflow-y-auto">
              {notifications.map((notification) => (
                <li key={notification.id}>
                  <button
                    type="button"
                    onClick={() => void markAsRead(notification)}
                    disabled={markingId !== null}
                    className={`w-full border-b border-[#F0EAD6] px-4 py-3 text-left transition hover:bg-[#FFFBE9] disabled:opacity-60 ${
                      notification.read ? "" : "bg-[#FFFBE9]/70"
                    }`}
                  >
                    <span className="flex items-start justify-between gap-3">
                      <span className="font-medium text-[#1A1A1A]">
                        {notification.title}
                      </span>
                      {!notification.read && (
                        <span
                          aria-label="Unread"
                          className="mt-1.5 size-2 shrink-0 rounded-full bg-red-600"
                        />
                      )}
                    </span>
                    <span className="mt-1 block text-sm text-gray-600">
                      {notification.message}
                    </span>
                    <time
                      dateTime={notification.createdAt}
                      className="mt-2 block text-xs text-gray-500"
                    >
                      {new Date(notification.createdAt).toLocaleString()}
                    </time>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>
      )}
    </div>
  );
}
