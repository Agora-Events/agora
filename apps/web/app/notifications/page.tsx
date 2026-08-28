"use client";

import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { useState } from "react";
import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";

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

  const unreadCount = notifications.filter((n) => !n.read).length;

  const markAllAsRead = () => {
    if (unreadCount === 0) return;
    setNotifications(notifications.map((n) => ({ ...n, read: true })));
    toast.success("All notifications marked as read");
  };

  const clearNotifications = () => {
    setNotifications([]);
  };

  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />
      <div className="flex-1 w-full max-w-[800px] mx-auto px-4 py-12 md:py-20">
        <div className="flex items-center justify-between mb-8">
          <div className="flex items-center gap-3">
            <h1 className="text-4xl font-extrabold italic text-ink-deep">Notifications</h1>
            {unreadCount > 0 && (
              <span
                data-testid="unread-badge"
                aria-label={`${unreadCount} unread notification${unreadCount === 1 ? "" : "s"}`}
                className="inline-flex items-center justify-center min-w-[28px] h-7 px-2 rounded-full bg-accent text-black text-sm font-bold border-2 border-black"
              >
                {unreadCount}
              </span>
            )}
          </div>
          {notifications.length > 0 && (
            <div className="flex gap-4">
              <button
                onClick={markAllAsRead}
                disabled={unreadCount === 0}
                aria-label="Mark all notifications as read"
                className="text-sm font-semibold text-ink-deep hover:underline disabled:text-gray-400 disabled:no-underline disabled:cursor-not-allowed"
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
          <div className="bg-white p-16 rounded-[32px] border-2 border-black shadow-[-8px_8px_0_rgba(0,0,0,1)] text-center flex flex-col items-center">
            <div className="w-24 h-24 bg-surface rounded-full flex items-center justify-center mb-6 border-2 border-black">
              <Image src="/icons/notification.svg" alt="No notifications" width={48} height={48} />
            </div>
            <h2 className="text-3xl font-bold italic mb-4">You&apos;re all caught up!</h2>
            <p className="text-ink-deep/70 mb-8 max-w-md mx-auto">
              You don&apos;t have any new notifications at the moment. Check back later for updates
              on your events and tickets.
            </p>
            <Link href="/discover">
              <Button
                backgroundColor="bg-accent"
                textColor="text-black"
                shadowColor="rgba(253,218,35,0.4)"
                className="px-8 font-bold text-lg"
              >
                Discover Events
              </Button>
            </Link>
          </div>
        )}
      </div>
      <Footer />
    </main>
  );
}
