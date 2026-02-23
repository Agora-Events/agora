"use client";

import Image from "next/image";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { dataEvents } from "@/components/events/mockups";
import { RegistrationBox } from "@/components/events/registration-box";
import { notFound } from "next/navigation";
import React, { use } from "react";

export default function EventDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = use(params);
  const eventId = parseInt(id);
  const event = dataEvents.find((e) => e.id === eventId);

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
            <RegistrationBox
              isFree={isFree}
              price={event.price}
              host={host}
            />

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
