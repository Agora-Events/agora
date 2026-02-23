"use client";

import Image from "next/image";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { dataEvents } from "@/components/events/mockups";
import { notFound } from "next/navigation";
import React, { useState, use } from "react";

export default function EventDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = use(params);
  const eventId = parseInt(id);
  const event = dataEvents.find((e) => e.id === eventId);
  const [quantity, setQuantity] = useState(1);

  if (!event) {
    notFound();
  }

  const isFree = event.price.toLowerCase() === "free";

  // Mock host data matching Figma
  const host = {
    name: "Stellar Community",
    avatar: "/icons/stellar-logo.svg",
    handle: "Daniel James",
    hostPfp: "/images/pfp.png",
  };

  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />

      <div className="flex-1 w-full max-w-[1221px] mx-auto px-6 py-6 sm:py-12">
        <div className="flex flex-col lg:flex-row gap-8 lg:gap-16">
          {/* LEFT COLUMN (Desktop) / TOP ITEMS (Mobile) */}
          <div className="lg:w-[55%] flex flex-col gap-8 lg:gap-10">
            {/* Cover Image Container - Dark Navy Background */}
            <div className="relative aspect-[16/10] sm:aspect-[16/11] w-full rounded-[32px] sm:rounded-[40px] overflow-hidden bg-[#0B151F] shadow-sm flex items-center justify-center p-6 sm:p-12">
              <div className="relative w-full h-full">
                <Image
                  src={event.imageUrl}
                  alt={event.title}
                  fill
                  className="object-contain"
                  priority
                />
              </div>
            </div>

            {/* Hosted By - Positioned after image on mobile */}
            <div className="flex flex-col gap-4">
              <h2 className="text-xl font-bold text-black font-heading">
                Hosted By
              </h2>
              <div className="flex items-center gap-3">
                <div className="relative w-8 h-8 rounded-full border border-black overflow-hidden bg-white">
                  <Image
                    src={host.avatar}
                    fill
                    alt="Stellar"
                    className="object-contain p-1.5"
                  />
                </div>
                <span className="text-[17px] font-medium text-black">
                  by <span className="italic">{host.name}</span>
                </span>
              </div>
            </div>

            {/* Desktop-only Map (Hidden on mobile) */}
            <div className="hidden lg:flex flex-col gap-6">
              <div className="flex items-center gap-4">
                <div className="w-10 h-10 rounded-full border border-black flex items-center justify-center">
                  <Image
                    src="/icons/location.svg"
                    width={20}
                    height={20}
                    alt="location"
                  />
                </div>
                <h2 className="text-xl font-bold text-black font-heading">
                  Location
                </h2>
              </div>
              <p className="text-[18px] font-medium text-black -mt-2">
                {event.location}
              </p>
              <div className="relative aspect-[16/10] w-full rounded-[24px] overflow-hidden border border-black/10">
                <Image
                  src="/images/map-placeholder.png"
                  alt="Location Map"
                  fill
                  className="object-cover"
                />
              </div>
            </div>
          </div>

          {/* RIGHT COLUMN (Desktop) / BOTTOM ITEMS (Mobile) */}
          <div className="lg:w-[45%] flex flex-col gap-8 lg:gap-10">
            {/* Title */}
            <h1 className="text-[36px] sm:text-[56px] font-bold leading-[1.1] text-black font-heading">
              {event.title}
            </h1>

            {/* Details (Location & Date) */}
            <div className="flex flex-col gap-6">
              <div className="flex items-center gap-4">
                <div className="w-11 h-11 rounded-full border border-black flex items-center justify-center shrink-0">
                  <Image
                    src="/icons/location.svg"
                    width={22}
                    height={22}
                    alt="location"
                  />
                </div>
                <span className="text-[18px] sm:text-[19px] font-medium text-black">
                  {event.location}
                </span>
              </div>
              <div className="flex items-center gap-4">
                <div className="w-11 h-11 rounded-full border border-black flex items-center justify-center shrink-0">
                  <Image
                    src="/icons/notification.svg"
                    width={22}
                    height={22}
                    alt="calendar"
                  />
                </div>
                <span className="text-[18px] sm:text-[19px] font-medium text-black">
                  {event.date}
                </span>
              </div>
            </div>

            {/* Registration Box */}
            <div className="bg-[#FFEFD3] rounded-[24px] p-6 sm:p-8 flex flex-col gap-8 relative overflow-hidden border border-black/5 shadow-sm">
              <div className="flex justify-between items-center z-10 flex-wrap gap-4">
                <div className="bg-white rounded-full px-6 py-2.5 italic text-gray-400 font-medium text-[17px] sm:text-[20px] shadow-sm flex-1 min-w-[150px]">
                  Registration
                </div>
                <div className="flex items-center gap-3">
                  <button
                    onClick={() => setQuantity(Math.max(1, quantity - 1))}
                    className="w-10 h-10 sm:w-12 sm:h-12 rounded-full bg-white border border-black/5 shadow-sm flex items-center justify-center hover:bg-gray-50 transition-colors text-2xl font-light text-black"
                  >
                    −
                  </button>
                  <div className="w-10 h-10 sm:w-12 sm:h-12 rounded-full bg-white border border-black/5 shadow-sm flex items-center justify-center">
                    <span className="text-lg sm:text-xl font-bold text-black">
                      {quantity}
                    </span>
                  </div>
                  <button
                    onClick={() => setQuantity(quantity + 1)}
                    className="w-10 h-10 sm:w-12 sm:h-12 rounded-full bg-white border border-black/5 shadow-sm flex items-center justify-center hover:bg-gray-50 transition-colors text-2xl font-light text-black"
                  >
                    +
                  </button>
                </div>
              </div>

              <p className="text-[16px] sm:text-[19px] text-black font-medium z-10">
                Welcome! To join the event, please register below.
              </p>

              <div className="flex items-center justify-between z-10 gap-4 flex-wrap">
                <button className="bg-[#FDDA23] text-black font-bold text-[18px] sm:text-[22px] h-14 sm:h-16 px-8 sm:px-10 rounded-full border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] active:translate-x-[2px] active:translate-y-[2px] active:shadow-none flex items-center gap-4">
                  <Image
                    src="/icons/dollar-circle.svg"
                    width={28}
                    height={28}
                    alt="dollar"
                  />
                  {isFree
                    ? "Free"
                    : `$${(parseFloat(event.price) * quantity).toFixed(2)}`}
                </button>
                <div className="flex items-center gap-3">
                  <div className="relative w-11 h-11 sm:w-14 sm:h-14 rounded-full border-2 border-black overflow-hidden bg-white shadow-[3px_3px_0px_0px_rgba(0,0,0,1)]">
                    <Image
                      src={host.hostPfp}
                      fill
                      alt={host.handle}
                      className="object-cover"
                    />
                  </div>
                  <span className="text-[16px] sm:text-[18px] italic font-medium text-black whitespace-nowrap">
                    {host.handle}
                  </span>
                </div>
              </div>

              <div className="absolute -right-8 -bottom-8 opacity-[0.06] scale-150 pointer-events-none rotate-12 z-0">
                <Image
                  src="/icons/stellar-logo.svg"
                  width={240}
                  height={240}
                  alt="bg-logo"
                />
              </div>
            </div>

            {/* About Section */}
            <div className="flex flex-col gap-6 pt-4">
              <h2 className="text-[20px] sm:text-[22px] font-bold text-black font-heading">
                About Event
              </h2>
              <div className="text-[16px] sm:text-[17px] text-black leading-relaxed font-normal flex flex-col gap-6">
                <p>
                  The Casa Stellar + Stellar Lab is an advanced, invitation-only
                  week-long builder residency and pro hackathon in Buenos Aires,
                  gathering top developers from across LATAM. This event is
                  designed to deepen loyalty and long-term commitment to the
                  Stellar ecosystem during DevConnect in Argentina. Unlike
                  introductory hackathons, this activation is designed for pro
                  builders: developers who have already engaged with Stellar
                  through past hackathons and the Stellar Ambassador program
                  across Latin America.
                </p>
                <div className="flex flex-col gap-2">
                  <p>
                    <span className="font-bold">Event:</span>{" "}
                    <span className="underline cursor-pointer hover:text-gray-700">
                      Stellar Asado
                    </span>
                  </p>
                  <p>
                    <span className="font-bold">Date:</span> November 17
                  </p>
                  <p>
                    <span className="font-bold">Time:</span> 6:00 PM - 9:00 PM
                  </p>
                  <p>
                    A builder-style kickoff to the residency with food, code and
                    real conversations with the ecosystem&apos;s top
                    contributors.
                  </p>
                </div>
                <div className="flex flex-col gap-2">
                  <p>
                    <span className="font-bold">Event:</span> Stellar Lab
                  </p>
                  <p className="underline cursor-pointer hover:text-gray-700">
                    Day 1: November 17 - The State of Stellar
                  </p>
                  <p className="underline cursor-pointer hover:text-gray-700">
                    Day 2: November 18 - Designing for Scale
                  </p>
                  <p className="underline cursor-pointer hover:text-gray-700">
                    Day 3: November 19 - From Vision to Execution
                  </p>
                </div>
              </div>
            </div>

            {/* Mobile-only Map (Hidden on desktop) */}
            <div className="lg:hidden flex flex-col gap-6 mt-8">
              <div className="flex items-center gap-4">
                <div className="w-10 h-10 rounded-full border border-black flex items-center justify-center">
                  <Image
                    src="/icons/location.svg"
                    width={20}
                    height={20}
                    alt="location"
                  />
                </div>
                <h2 className="text-xl font-bold text-black font-heading">
                  Location
                </h2>
              </div>
              <p className="text-[17px] font-medium text-black -mt-2">
                {event.location}
              </p>
              <div className="relative aspect-[16/10] w-full rounded-[24px] overflow-hidden border border-black/10">
                <Image
                  src="/images/map-placeholder.png"
                  alt="Location Map"
                  fill
                  className="object-cover"
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      <Footer />

      {/* Background Watermarks */}
      <div className="fixed -right-20 -bottom-20 opacity-[0.06] pointer-events-none -rotate-12 select-none z-0">
        <Image
          src="/icons/stellar-logo.svg"
          width={600}
          height={600}
          alt="bg-watermark"
        />
      </div>
    </main>
  );
}
