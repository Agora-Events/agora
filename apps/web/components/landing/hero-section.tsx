"use client";

import { Navbar } from "@/components/layout/navbar";
import { Button } from "@/components/ui/button";
import { motion } from "framer-motion";
import Image from "next/image";
import Link from "next/link";
import { useState, useId } from "react";

/**
 * Hero section component for the landing page
 */
export function HeroSection() {
  return (
    <div
      className="relative w-full min-h-screen flex flex-col items-center bg-cover bg-center bg-no-repeat select-none overflow-hidden"
      style={{ backgroundImage: 'url("/backgrounds/hero-gradient.svg")' }}
    >
      <Navbar />

      <div className="flex-1 flex flex-col items-center pt-[52px] w-full max-w-[1200px] mx-auto px-4">
        <div className="flex items-center justify-center gap-2 w-[190px] h-[34px] bg-[rgba(0,0,0,0.18)] border border-[rgba(0,0,0,0.2)] rounded-full mb-8">
          <Image
            src="/icons/stellar-logo.svg"
            alt="Stellar"
            width={16}
            height={16}
          />
          <span className="text-sm font-medium">Powered by stellar</span>
        </div>

        {/* Hero Heading */}
        <h1 className="text-4xl md:text-[58px] leading-tight md:leading-[66px] font-semibold italic text-center text-black mb-6 md:mb-8 transition-all">
          <div>Plan Events. Bring People Together.</div>
          <div className="text-[#CAAE1C]">Grow Communities.</div>
        </h1>
        <p className="text-lg md:text-[20px] font-light text-center max-w-[800px] mb-8 px-4">
          Create your event page, invite your community, and sell tickets
          seamlessly.
          <br className="hidden md:block" />
          Start hosting today.
        </p>

        {/* Buttons */}
        <div className="flex flex-col sm:flex-row items-center gap-4 mb-16 md:mb-20 w-full sm:w-auto px-4 sm:px-0">
          <Link href="/create-event" className="w-full sm:w-auto">
            <Button
              className="w-full sm:w-[215px] h-[56px]"
              backgroundColor="bg-white"
              textColor="text-black"
              shadowColor="rgba(0,0,0,1)"
            >
              <span>Create Your Event</span>
              <Image
                src="/icons/arrow-up-right-01.svg"
                alt="Arrow"
                width={20}
                height={20}
              />
            </Button>
          </Link>

          <Link href="/pricing" className="w-full sm:w-auto">
            <Button
              className="w-full sm:w-[135px] h-[56px]"
              backgroundColor="bg-[#FDDA23]"
              textColor="text-white"
              shadowColor="rgba(0,0,0,1)"
            >
              <span>Learn More</span>
            </Button>
          </Link>
        </div>

        <div className="relative w-full max-w-[1000px] flex justify-center mt-auto">
          <Image
            src="/images/World.png"
            alt="World Map"
            width={1000}
            height={500}
            className="object-contain"
            priority
          />

          {/* Tooltips */}
          {/* Organizers */}
          <Tooltip
            icon="/icons/Organizers.png"
            label="Organizers"
            className="md:top-[30%] top-[20%] left-[57%] md:left-[58%]"
            delay={0.2}
          />

          {/* Meet ups */}
          <Tooltip
            icon="/icons/MeetUps.png"
            label="Meetups"
            className=" md:top-[70%] top-[60%] -right-[4%] md:-right-[2%]"
            delay={0.4}
          />

          {/* Party */}
          <Tooltip
            icon="/icons/Party.png"
            label="Parties"
            className="bottom-[35%] left-[2%] md:left-[11%]"
            delay={0.6}
          />

          {/* Events */}
          <Tooltip
            icon="/icons/Events.png"
            label="Events"
            className="md:top-[5%] -top-[10%] right-[5%] md:right-[15%]"
            delay={0.8}
          />
        </div>
      </div>
    </div>
  );
}

function Tooltip({
  icon,
  className,
  delay,
  label,
}: {
  icon: string;
  className: string;
  delay: number;
  label: string;
}) {
  const [isFocused, setIsFocused] = useState(false);
  const id = useId();

  return (
    <motion.div
      id={id}
      initial={{ scale: 0, opacity: 0, y: 10 }}
      whileInView={{ scale: 1, opacity: 1, y: 0 }}
      animate={isFocused ? { scale: 1.1 } : {}}
      whileHover={{ scale: 1.1 }}
      viewport={{ once: false, margin: "-50px" }}
      transition={{
        delay,
        type: "spring",
        stiffness: 260,
        damping: 20,
      }}
      className={`absolute ${className} z-20 w-16 h-16 md:w-24 md:h-24 outline-none focus-visible:ring-4 focus-visible:ring-[#FDDA23] rounded-full cursor-pointer transition-shadow`}
      tabIndex={0}
      role="tooltip"
      aria-label={label}
      onFocus={() => setIsFocused(true)}
      onBlur={() => setIsFocused(false)}
    >
      <Image
        src={icon}
        alt="" // Alt is empty because aria-label is on the wrapper
        fill
        className="drop-shadow-xl object-contain"
      />
    </motion.div>
  );
}
