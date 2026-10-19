"use client";

import { motion, useReducedMotion } from "framer-motion";
import Image from "next/image";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import EmptyStateBg from "@/public/icons/empty-state-bg.svg";
import ZeroIcon from "@/public/icons/zero.svg";

export function UpcomingEventsEmptyState() {
  const router = useRouter();
  const shouldReduceMotion = useReducedMotion();

  return (
    <motion.div
      initial={shouldReduceMotion ? { opacity: 0 } : { opacity: 0, y: 10 }}
      animate={shouldReduceMotion ? { opacity: 1 } : { opacity: 1, y: 0 }}
      transition={{ duration: 0.4, ease: "easeOut" }}
      className="mx-auto flex min-h-[29rem] w-full max-w-[32rem] flex-col items-center justify-center px-4 text-center sm:px-6"
    >
      <div className="flex w-full max-w-[19rem] flex-col items-center rounded-[2rem] bg-[#FFEFD3] px-6 py-8 shadow-[0_24px_50px_rgba(255,219,159,0.32)] sm:max-w-[20rem] sm:px-7 sm:py-9">
        <div className="relative flex items-start justify-center">
          <div className="flex h-[12.5rem] w-[12.5rem] items-center justify-center rounded-[1.75rem] bg-white px-5 py-5 shadow-[0_16px_32px_rgba(44,32,19,0.08)] sm:h-[14rem] sm:w-[14rem]">
            <Image
              src={EmptyStateBg}
              alt=""
              aria-hidden
              width={224}
              height={224}
              className="h-full w-full object-contain"
            />
          </div>

          <div className="absolute -right-6 -top-4 flex h-[4.25rem] w-[4.25rem] items-center justify-center rounded-[1.5rem] bg-white p-3 shadow-[0_12px_24px_rgba(44,32,19,0.08)] sm:-right-8 sm:-top-6 sm:h-[5.25rem] sm:w-[5.25rem]">
            <Image
              src={ZeroIcon}
              alt=""
              aria-hidden
              width={48}
              height={48}
              className="h-full w-full object-contain"
            />
          </div>
        </div>

        <h3 className="mt-6 text-[1.25rem] font-medium leading-tight text-[#131517] sm:text-[1.375rem]">
          Nothing Here, Yet
        </h3>
      </div>

      <p className="mt-6 max-w-[24rem] text-sm leading-6 text-black/60 sm:text-base">
        You don&apos;t have any upcoming events right now. Explore what&apos;s
        happening and grab a ticket for your next event.
      </p>

      <Button
        type="button"
        onClick={() => router.push("/discover")}
        backgroundColor="bg-[#FFD21F]"
        textColor="text-black"
        className="mt-5 w-full max-w-[14rem] px-5 py-3 text-sm sm:w-auto sm:text-base"
        aria-label="Discover events"
      >
        <span>Discover Events</span>
        <Image
          src="/icons/arrow-right.svg"
          alt=""
          aria-hidden
          width={18}
          height={18}
          className="h-[18px] w-[18px] object-contain"
        />
      </Button>
    </motion.div>
  );
}
