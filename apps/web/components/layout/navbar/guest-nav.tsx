"use client";

import Link from "next/link";
import Image from "next/image";
import { Button } from "@/components/ui/button";
import { useTheme } from "@/components/providers/theme-context";
import { BaseNav, type NavItem } from "./base-nav";

const GUEST_NAV_ITEMS: NavItem[] = [
  {
    href: "/discover",
    icon: "/icons/earth.svg",
    text: "Discover Events",
    isActive: (p) => p === "/discover" || p.startsWith("/events"),
  },
  {
    href: "/pricing",
    icon: "/icons/dollar-circle.svg",
    text: "Pricing",
    isActive: (p) => p === "/pricing",
  },
  {
    href: "/stellar",
    icon: "/icons/stellar-xlm-logo 1.svg",
    text: "Stellar Ecosystem",
    isActive: (p) => p === "/stellar",
  },
  {
    href: "/faqs",
    icon: "/icons/help-circle.svg",
    text: "FAQs",
    isActive: (p) => p === "/faqs",
  },
];

const guestCta = (
  <Link href="/auth" title="Sign in to create an event">
    <Button
      backgroundColor="bg-white"
      textColor="text-black"
      shadowColor="rgba(0,0,0,1)"
      aria-label="Create event - sign in required"
    >
      <Image
        src="/icons/lock.svg"
        alt=""
        width={18}
        height={18}
        aria-hidden="true"
      />
      <span>Create Your Event</span>
      <Image
        src="/icons/arrow-up-right-01.svg"
        alt=""
        width={24}
        height={24}
        aria-hidden="true"
        className="group-hover:translate-x-0.5 group-hover:-translate-y-0.5 transition-transform"
      />
    </Button>
  </Link>
);

export function GuestNav({ pathname }: { pathname: string }) {
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
      isAuthenticated={false}
      navItems={GUEST_NAV_ITEMS}
      ctaSlot={guestCta}
      endSlot={
        <>
          {themeToggle}
        </>
      }
    />
  );
}
