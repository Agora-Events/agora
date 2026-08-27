"use client";

import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { EmptyState } from "@/components/ui/empty-state";
import { useState } from "react";
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

export default function NotificationsPage() {
  const [notifications, setNotifications] = useState<NotificationItem[]>(mockNotifications);

  const markAllAsRead = () => {
    setNotifications(notifications.map((n) => ({ ...n, read: true })));
  };

  const clearNotifications = () => {
    setNotifications([]);
  };

  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />
      <div className="flex-1 w-full max-w-[800px] mx-auto px-4 py-12 md:py-20">
        <div className="flex items-center justify-between mb-8">
          <h1 className="text-4xl font-extrabold italic text-ink-deep">Notifications</h1>
          {notifications.length > 0 && (
            <div className="flex gap-4">
              <button
                onClick={markAllAsRead}
                className="text-sm font-semibold text-ink-deep hover:underline"
              >
                Mark all as read
              </button>
              <button
                onClick={clearNotifications}
                className="text-sm font-semibold text-error hover:underline"
              >
                Clear all
              </button>
            </div>
          )}
        </div>

        {notifications.length > 0 ? (
          <div className="flex flex-col gap-4">
            {notifications.map((notification) => (
              <div
                key={notification.id}
                className={`flex gap-4 p-6 rounded-2xl border-2 border-black transition-all ${
                  notification.read
                    ? "bg-white shadow-[-4px_4px_0_rgba(0,0,0,1)] opacity-70"
                    : "bg-surface shadow-[-6px_6px_0_rgba(0,0,0,1)] hover:-translate-y-1"
                }`}
              >
                <div className="w-12 h-12 shrink-0 bg-white rounded-full flex items-center justify-center border-2 border-black">
                  <Image
                    src={
                      notification.type === "ticket_confirmation"
                        ? "/icons/ticket.svg"
                        : notification.type === "event_reminder"
                          ? "/icons/calendar.svg"
                          : "/icons/user-group.svg"
                    }
                    alt={notification.type}
                    width={24}
                    height={24}
                  />
                </div>
                <div className="flex-1">
                  <div className="flex justify-between items-start mb-1">
                    <h3 className="font-bold text-lg text-ink-deep">{notification.title}</h3>
                    <span className="text-xs font-semibold text-gray-500 whitespace-nowrap">
                      {notification.time}
                    </span>
                  </div>
                  <p className="text-ink-deep/80 mb-3">{notification.message}</p>
                  {notification.link && (
                    <Link href={notification.link}>
                      <span className="text-sm font-bold underline hover:text-accent transition-colors">
                        View details
                      </span>
                    </Link>
                  )}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <EmptyState
            icon={
              <Image
                src="/icons/notification.svg"
                alt="No notifications"
                width={40}
                height={40}
                className="opacity-70"
              />
            }
            title="You're all caught up!"
            description="You don't have any new notifications at the moment. Check back later for updates on your events and tickets."
            action={{ label: "Discover Events", href: "/discover" }}
          />
        )}
      </div>
      <Footer />
    </main>
  );
}
