"use client";

import Link from "next/link";
import Image from "next/image";
import { Button } from "@/components/ui/button";
import { NavLink } from "./nav-link";

export function UserNav({ pathname }: { pathname: string }) {
  return (
    <div className="flex items-center justify-between w-full">
      <div className="flex items-center gap-8 lg:gap-[137px]">
        <Link href="/" className="flex items-center z-50">
          <Image
            src="/logo/agora logo.svg"
            alt="Agora Logo"
            width={100}
            height={30}
            className="h-auto w-auto"
          />
        </Link>

        <div className="hidden lg:flex items-center gap-[53px]">
          <div className="flex items-center gap-6">
            <NavLink
              href="/"
              icon="/icons/home.svg"
              text="Home"
              isActive={pathname === "/"}
            />
            <NavLink
              href="/discover"
              icon="/icons/earth-yellow.svg"
              text="Discover Events"
              isActive={pathname === "/discover"}
            />
            <NavLink
              href="/organizers"
              icon="/icons/user-group.svg"
              text="Organizers"
              isActive={pathname === "/organizers"}
            />
            <NavLink
              href="/stellar"
              icon="/icons/stellar-xlm-logo 1.svg"
              text="Stellar Ecosystem"
              isActive={pathname === "/stellar"}
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

      <div className="hidden md:flex items-center gap-4 lg:gap-[29px]">
        <Button
          backgroundColor="bg-white"
          className="relative w-[55.22px] h-[53px] px-[10px] py-[10px]"
          textColor="text-black"
          shadowColor="rgba(0,0,0,1)"
        >
          <div className="size-[9px] bg-red-500 rounded-full absolute top-[4px] right-[2px]" />
          <Image
            src="/icons/notification.svg"
            alt="Arrow"
            width={24}
            height={24}
            className="group-hover:translate-x-0.5 group-hover:-translate-y-0.5 transition-transform"
          />
        </Button>
        <Button
          backgroundColor="bg-white"
          className="relative w-[55.22px] h-[53px] !px-0 py-0"
          textColor="text-black"
          shadowColor="rgba(0,0,0,1)"
        >
          <div className=" size-[49px] rounded-full">
            <Image
              src="/images/pfp.png"
              alt="Arrow"
              width={49}
              height={49}
              className="group-hover:translate-x-0.5 group-hover:-translate-y-0.5 transition-transform"
            />
          </div>
        </Button>
      </div>
    </div>
  );
}
