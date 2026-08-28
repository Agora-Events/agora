"use client";

import { Suspense, useEffect, useState } from "react";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { ProfileSidebar } from "@/components/profile/profile-sidebar";
import { FollowButton } from "@/components/profile/follow-button";
import { OrganizerProfileSkeleton } from "@/components/profile/organizer-profile-skeleton";
import { EventCard } from "@/components/events/event-card";
import Image from "next/image";
import Link from "next/link";
import useSWR from "swr";

const fetcher = (url: string) => fetch(url).then((res) => res.json());

function EmptyState({ icon, heading, subtext }: { icon: React.ReactNode; heading: string; subtext: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 px-6 text-center">
      <div className="w-20 h-20 rounded-full bg-surface flex items-center justify-center mb-5 border-2 border-black shadow-[-4px_4px_0_rgba(0,0,0,1)]">
        {icon}
      </div>
      <h3 className="text-ink-soft font-bold text-xl mb-2 italic">{heading}</h3>
      <p className="text-gray-500 text-sm max-w-xs mb-6">{subtext}</p>
      <Link
        href="/discover"
        className="inline-flex items-center gap-2 bg-accent text-black font-bold px-5 py-2.5 rounded-full border-2 border-black hover:shadow-[-4px_4px_0_rgba(0,0,0,1)] hover:-translate-y-0.5 transition-all"
      >
        Explore Events
      </Link>
    </div>
  );
}

const CalendarIcon = () => (
  <Image src="/icons/calendar.svg" width={32} height={32} alt="Calendar" className="invert" />
);

// Types for events (matching standard API response if possible, using any for now or adapting from home page)
interface EventItem {
  id: number;
  title: string;
  date: string;
  location: string;
  price: string;
  imageUrl: string;
  organizer_wallet?: string;
  start_time?: string;
}

function OrganizerEvents({ address }: { address: string }) {
  const { data: eventsData, error, isLoading } = useSWR<{ events: EventItem[] }>("/api/v1/events", fetcher);

  const hostedEvents = eventsData?.events?.filter((e) => e.organizer_wallet === address) || [];

  return (
    <section className="bg-white rounded-3xl border-2 border-black shadow-[-8px_8px_0_rgba(0,0,0,1)] overflow-hidden">
      <div className="px-8 pt-8 pb-4 border-b-2 border-black bg-surface">
        <h2 className="text-2xl font-bold italic text-ink-deep">Hosted Events</h2>
        <p className="text-sm text-gray-600 mt-1">Events organized by this creator</p>
      </div>
      
      {isLoading ? (
        <div className="p-8 flex flex-col gap-5">
          <div className="h-32 bg-gray-200 animate-pulse rounded-2xl border-2 border-black" />
          <div className="h-32 bg-gray-200 animate-pulse rounded-2xl border-2 border-black" />
        </div>
      ) : hostedEvents.length > 0 ? (
        <div className="p-8 flex flex-col gap-6" data-testid="hosted-events-list">
          {hostedEvents.map((event) => (
            <EventCard key={event.id} {...event as any} />
          ))}
        </div>
      ) : (
        <div data-testid="hosted-empty-state">
          <EmptyState
            icon={<CalendarIcon />}
            heading="No hosted events yet"
            subtext="This organizer hasn't created any public events at this time."
          />
        </div>
      )}
    </section>
  );
}

export default function OrganizerProfilePage({ params }: { params: { id: string } }) {
  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />
      <div
        className="flex-1"
        aria-busy="true"
        aria-label="Loading organizer profile"
        data-testid="organizer-profile-loading"
      >
        <Suspense fallback={<OrganizerProfileSkeleton />}>
          <div className="flex-1 w-full max-w-6xl mx-auto px-4 py-10 md:py-20">
            <div className="flex flex-col md:flex-row gap-10 items-start">
              <div className="w-full md:w-[32%] md:sticky md:top-24 flex flex-col gap-4">
                <ProfileSidebar address={params.id} />
                <FollowButton organizerId={params.id} />
              </div>

              <div className="flex-1 flex flex-col gap-10 w-full">
                <OrganizerEvents address={params.id} />
              </div>
            </div>
          </div>
        </Suspense>
      </div>
      <Footer />
    </main>
  );
}
