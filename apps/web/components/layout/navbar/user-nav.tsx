"use client";

import { useState } from "react";
import Link from "next/link";
import Image from "next/image";
import { Button } from "@/components/ui/button";
import { BaseNav, type NavItem } from "./base-nav";
import { useStellarNetwork } from "@/hooks/useStellarNetwork";

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
    href: "/wallet",
    icon: "/icons/ticket.svg",
    text: "My Wallet",
    isActive: (p) => p === "/wallet",
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

/**
 * Network status pill — Issue #1491
 *
 * Displays a small badge next to the wallet avatar reflecting whether the
 * connected Freighter wallet is on the correct network.  Clicking it when
 * on the wrong network fires requestNetworkSwitch().
 *
 * Only visible when NEXT_PUBLIC_STELLAR_NETWORK === "TESTNET".
 */
function NetworkPill() {
  const { isCorrectNetwork, currentNetwork, isLoading, requestNetworkSwitch } =
    useStellarNetwork();

  if (process.env.NEXT_PUBLIC_STELLAR_NETWORK !== "TESTNET") return null;
  if (isLoading) return null;

  const label = isCorrectNetwork ? "Testnet" : "Wrong Network";
  const title = isCorrectNetwork
    ? `Connected to Stellar Testnet (${currentNetwork ?? ""})`
    : `Wallet is on "${currentNetwork ?? "unknown"}" — click to switch`;

  return (
    <button
      type="button"
      title={title}
      aria-label={title}
      onClick={isCorrectNetwork ? undefined : requestNetworkSwitch}
      className={`inline-flex items-center gap-1 text-xs font-semibold px-2 py-0.5 rounded-full border transition-colors ${
        isCorrectNetwork
          ? "bg-green-50 border-green-300 text-green-700 cursor-default"
          : "bg-red-50 border-red-300 text-red-700 cursor-pointer hover:bg-red-100"
      }`}
    >
      <span
        className={`w-1.5 h-1.5 rounded-full ${
          isCorrectNetwork ? "bg-green-500" : "bg-red-500"
        }`}
        aria-hidden="true"
      />
      {label}
    </button>
  );
}

function UserEndSlot() {
  const [notifications] = useState<any[]>([]);

  return (
    <>
      {/* Network status pill — only visible on testnet builds */}
      <NetworkPill />

      <div className="relative">
        <Link href="/notifications">
          <Button
            backgroundColor="bg-white"
            className="relative w-[55.22px] h-[53px] px-[10px] py-[10px]"
            textColor="text-black"
            shadowColor="rgba(0,0,0,1)"
            aria-label="View notifications"
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
        </Link>
      </div>

      <Link href="/profile">
        <Button
          backgroundColor="bg-white"
          className="relative w-[55.22px] h-[53px] px-0! py-0"
          textColor="text-black"
          shadowColor="rgba(0,0,0,1)"
          aria-label="User profile"
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
