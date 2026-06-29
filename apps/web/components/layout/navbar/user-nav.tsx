"use client";

import { useState } from "react";
import Link from "next/link";
import Image from "next/image";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { BaseNav, type NavItem } from "./base-nav";

const USER_NAV_ITEMS: NavItem[] = [
  {
    href: "/",
    icon: "/icons/home.svg",
    text: "Home",
    isActive: (p) => p === "/",
  },
  {
    href: "/discover",
    icon: "/icons/earth-yellow.svg",
    text: "Discover Events",
    isActive: (p) => p === "/discover" || p.startsWith("/events"),
  },
  {
    href: "/organizers",
    icon: "/icons/user-group.svg",
    text: "Organizers",
    isActive: (p) => p === "/organizers",
  },
  {
    href: "/stellar",
    icon: "/icons/stellar-xlm-logo 1.svg",
    text: "Stellar Ecosystem",
    isActive: (p) => p === "/stellar",
  },
];

interface Notification {
  id: string;
  message: string;
  time: string;
  read: boolean;
}

function NotificationsPanel({ notifications }: { notifications: Notification[] }) {
  return (
    <div className="absolute top-full right-0 mt-2 w-80 bg-white rounded-2xl border border-border-warm shadow-[-4px_4px_0_rgba(0,0,0,1)] overflow-hidden z-50">
      <div className="flex items-center justify-between px-5 py-4 border-b border-border-warm">
        <h3 className="font-semibold text-ink-deep text-sm">Notifications</h3>
        {notifications.length > 0 && (
          <span className="text-xs text-muted-text">{notifications.length} new</span>
        )}
      </div>

      {notifications.length > 0 ? (
        <ul className="divide-y divide-border-warm max-h-72 overflow-y-auto">
          {notifications.map((n) => (
            <li key={n.id} className={`px-5 py-3 ${n.read ? "opacity-60" : ""}`}>
              <p className="text-sm text-ink-deep">{n.message}</p>
              <p className="text-xs text-muted-text mt-0.5">{n.time}</p>
            </li>
          ))}
        </ul>
      ) : (
        <EmptyState
          icon={
            <Image
              src="/icons/notification.svg"
              width={32}
              height={32}
              alt="no notifications"
            />
          }
          title="No notifications"
          description="You're all caught up! Check back later for updates on events you follow."
        />
      )}
    </div>
  );
}

const userCta = (
  <Link href="/create-event">
    <Button
      backgroundColor="bg-white"
      textColor="text-black"
      shadowColor="rgba(0,0,0,1)"
    >
      <span>Create Your Event</span>
      <Image
        src="/icons/arrow-up-right-01.svg"
        alt="Arrow"
        width={24}
        height={24}
        className="group-hover:translate-x-0.5 group-hover:-translate-y-0.5 transition-transform"
      />
    </Button>
  </Link>
);

function UserEndSlot() {
  const [isOpen, setIsOpen] = useState(false);
  // Empty notifications list — triggers the EmptyState
  const notifications: Notification[] = [];

  return (
    <>
      <div className="relative">
        <Button
          backgroundColor="bg-white"
          className="relative w-[55.22px] h-[53px] px-[10px] py-[10px]"
          textColor="text-black"
          shadowColor="rgba(0,0,0,1)"
          onClick={() => setIsOpen((prev) => !prev)}
          aria-label="Toggle notifications"
          aria-expanded={isOpen}
          aria-haspopup="true"
        >
          {notifications.length > 0 && (
            <div className="size-[9px] bg-red-500 rounded-full absolute top-[4px] right-[2px]" />
          )}
          <Image
            src="/icons/notification.svg"
            alt="Notifications"
            width={24}
            height={24}
            className="group-hover:translate-x-0.5 group-hover:-translate-y-0.5 transition-transform"
          />
        </Button>

        {isOpen && <NotificationsPanel notifications={notifications} />}
      </div>

      <Link href="/profile">
        <Button
          backgroundColor="bg-white"
          className="relative w-[55.22px] h-[53px] px-0! py-0"
          textColor="text-black"
          shadowColor="rgba(0,0,0,1)"
        >
          <div className="size-[49px] rounded-full">
            <Image
              src="/images/pfp.png"
              alt="Profile"
              width={49}
              height={49}
              className="group-hover:translate-x-0.5 group-hover:-translate-y-0.5 transition-transform"
            />
          </div>
        </Button>
      </Link>
    </>
  );
}

export function UserNav({ pathname }: { pathname: string }) {
  const { theme, toggleTheme } = useTheme();

  const themeToggle = (
    <Button
      backgroundColor="bg-white"
      className="relative w-[55.22px] h-[53px] px-[10px] py-[10px]"
      textColor="text-black"
      shadowColor="rgba(0,0,0,1)"
      onClick={toggleTheme}
      aria-label={theme === "light" ? "Switch to dark mode" : "Switch to light mode"}
    >
      {theme === "light" ? (
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" fill="currentColor" />
        </svg>
      ) : (
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <circle cx="12" cy="12" r="5" fill="currentColor" />
          <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
        </svg>
      )}
    </Button>
  );

  return (
    <BaseNav
      pathname={pathname}
      isAuthenticated={true}
      navItems={USER_NAV_ITEMS}
      ctaSlot={userCta}
      endSlot={<UserEndSlot />}
    />
  );
}
