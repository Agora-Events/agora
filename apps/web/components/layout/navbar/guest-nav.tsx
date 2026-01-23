"use client";

import Link from "next/link";
import Image from "next/image";
import { Button } from "@/components/ui/button";
import { NavLink } from "./nav-link";

export function GuestNav({ pathname }: { pathname: string }) {
  return (
    <div className="flex items-center gap-[231px] w-full">
      <Link href="/" className="flex items-center z-50">
        <Image
          src="/logo/agora logo.svg"
          alt="Agora Logo"
          width={100}
          height={30}
          className="h-auto w-auto"
        />
      </Link>

      <div className="hidden lg:flex items-center flex-1 gap-[170px]">
        <div className="flex items-center gap-[25px]">
          <NavLink
            href="/discover"
            icon="/icons/earth.svg"
            text="Discover Events"
            isActive={pathname === "/discover"}
          />
          <NavLink
            href="/pricing"
            icon="/icons/dollar-circle.svg"
            text="Pricing"
            isActive={pathname === "/pricing"}
          />
          <NavLink
            href="/stellar"
            icon="/icons/stellar-xlm-logo 1.svg"
            text="Stellar Ecosystem"
            isActive={pathname === "/stellar"}
          />
          <NavLink
            href="/faqs"
            icon="/icons/help-circle.svg"
            text="FAQs"
            isActive={pathname === "/faqs"}
          />
        </div>

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
      </div>
    </div>
  );
}
