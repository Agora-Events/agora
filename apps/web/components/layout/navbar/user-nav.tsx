"use client";

import Link from "next/link";
import Image from "next/image";
import { Button } from "@/components/ui/button";
import { NotificationDropdown } from "@/components/notifications/NotificationDropdown";
import { NavLink } from "./nav-link";

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
  return (
    <>
      {/* Testnet/Mainnet switcher — Issue #1541 */}
      <NetworkSwitcher />

      {/* Network status pill — only visible on testnet builds */}
      <NetworkPill />

      <div className="hidden md:flex items-center gap-4 lg:gap-[29px]">
        <NotificationDropdown />
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
