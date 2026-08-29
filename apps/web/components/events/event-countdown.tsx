"use client";

import { useEffect, useState } from "react";

interface EventCountdownProps {
  startsAt: string; // ISO date string
}

export function EventCountdown({ startsAt }: EventCountdownProps) {
  const [timeLeft, setTimeLeft] = useState<{
    days: number;
    hours: number;
    minutes: number;
    seconds: number;
    status: "upcoming" | "happening" | "ended";
  }>({
    days: 0,
    hours: 0,
    minutes: 0,
    seconds: 0,
    status: "upcoming",
  });

  useEffect(() => {
    const updateCountdown = () => {
      const now = new Date().getTime();
      const start = new Date(startsAt).getTime();
      const end = start + 2 * 60 * 60 * 1000; // start + 2 hours

      if (now >= end) {
        setTimeLeft({ days: 0, hours: 0, minutes: 0, seconds: 0, status: "ended" });
        return;
      }

      if (now >= start) {
        setTimeLeft({ days: 0, hours: 0, minutes: 0, seconds: 0, status: "happening" });
        return;
      }

      const diff = start - now;
      const days = Math.floor(diff / (1000 * 60 * 60 * 24));
      const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
      const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
      const seconds = Math.floor((diff % (1000 * 60)) / 1000);

      setTimeLeft({ days, hours, minutes, seconds, status: "upcoming" });
    };

    updateCountdown();
    const interval = setInterval(updateCountdown, 1000);

    return () => clearInterval(interval);
  }, [startsAt]);

  if (timeLeft.status === "happening") {
    return (
      <div className="text-[14px] sm:text-[16px] font-semibold text-accent" aria-live="off">
        Happening now
      </div>
    );
  }

  if (timeLeft.status === "ended") {
    return (
      <div className="text-[14px] sm:text-[16px] font-semibold text-gray-500" aria-live="off">
        Event ended
      </div>
    );
  }

  // Hide seconds when more than 24 hours remain
  const showSeconds = timeLeft.days === 0 && timeLeft.hours < 24;

  const parts = [];
  if (timeLeft.days > 0) parts.push(`${timeLeft.days}d`);
  if (timeLeft.hours > 0) parts.push(`${timeLeft.hours}h`);
  if (timeLeft.minutes > 0) parts.push(`${timeLeft.minutes}m`);
  if (showSeconds && timeLeft.seconds > 0) parts.push(`${timeLeft.seconds}s`);

  return (
    <div className="text-[14px] sm:text-[16px] font-semibold text-black" aria-live="off">
      {parts.join(" ")}
    </div>
  );
}
