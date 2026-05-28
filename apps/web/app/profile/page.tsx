"use client";

import { Suspense } from "react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { ProfileSidebar } from "@/components/profile/profile-sidebar";
import { EventCard } from "@/components/events/event-card";

type EventItem = {
  id: number;
  title: string;
  date: string;
  location: string;
  price: string;
  imageUrl: string;
};

const HOSTED_EVENTS: EventItem[] = [
  {
    id: 1,
    title: "Stellar Builders Summit",
    date: "Sat, Apr 12 · 10:00 AM",
    location: "San Francisco, CA",
    price: "25",
    imageUrl: "/images/event1.png",
  },
  {
    id: 2,
    title: "Web3 Community Meetup",
    date: "Thu, May 1 · 6:00 PM",
    location: "Discord",
    price: "Free",
    imageUrl: "/images/event2.png",
  },
];

const ATTENDED_EVENTS: EventItem[] = [
  {
    id: 3,
    title: "DeFi & Payments Workshop",
    date: "Fri, Mar 7 · 2:00 PM",
    location: "New York, NY",
    price: "Free",
    imageUrl: "/images/event3.png",
  },
];

function EmptyState({ icon, heading, subtext }: { icon: React.ReactNode; heading: string; subtext: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 px-6 text-center">
      <div className="w-20 h-20 rounded-full bg-surface flex items-center justify-center mb-5">
        {icon}
      </div>
      <h3 className="text-ink-soft font-semibold text-lg mb-2">{heading}</h3>
      <p className="text-gray-500 text-sm max-w-xs mb-6">{subtext}</p>
      <Link
        href="/events"
        className="inline-flex items-center gap-2 bg-ink-soft text-white text-sm font-medium px-5 py-2.5 rounded-full hover:bg-ink-soft transition-colors"
      >
        Explore Events
      </Link>
    </div>
  );
}

import Image from "next/image";

const CalendarIcon = () => (
  <Image src="/icons/calendar.svg" width={32} height={32} alt="Calendar" className="text-amber-400" />
);

const TicketIcon = () => (
  <Image src="/icons/ticket.svg" width={32} height={32} alt="Ticket" className="text-amber-400" />
);

function ProfileContent() {
  const searchParams = useSearchParams();
  const isEmpty = searchParams.get("empty") === "1";

  const hostedEvents = isEmpty ? [] : HOSTED_EVENTS;
  const attendedEvents = isEmpty ? [] : ATTENDED_EVENTS;

  return (
    <div className="flex-1 w-full max-w-6xl mx-auto px-4 py-10">
      <div className="flex flex-col md:flex-row gap-8 items-start">
        <div className="w-full md:w-[28%] md:sticky md:top-24">
          <ProfileSidebar />
        </div>

        <div className="flex-1 flex flex-col gap-6">
          {/* Hosting section */}
          <section className="bg-white rounded-2xl border border-border-warm shadow-sm overflow-hidden">
            <div className="px-6 pt-6 pb-4 border-b border-border-warm">
              <h2 className="text-lg font-semibold text-ink-soft">Hosting</h2>
              <p className="text-sm text-gray-500 mt-0.5">Events you&apos;re organizing</p>
            </div>
            {hostedEvents.length > 0 ? (
              <div className="p-6 flex flex-col gap-5" data-testid="hosted-events-list">
                {hostedEvents.map((event) => (
                  <EventCard key={event.id} {...event} />
                ))}
              </div>
            ) : (
              <div data-testid="hosted-empty-state">
                <EmptyState
                  icon={<CalendarIcon />}
                  heading="No hosted events yet"
                  subtext="You haven't created any public events. Start hosting and bring your community together."
                />
              </div>
            )}
          </section>

          {/* Attended section */}
          <section className="bg-white rounded-2xl border border-border-warm shadow-sm overflow-hidden">
            <div className="px-6 pt-6 pb-4 border-b border-border-warm">
              <h2 className="text-lg font-semibold text-ink-soft">Events</h2>
              <p className="text-sm text-gray-500 mt-0.5">Events you&apos;ve attended</p>
            </div>
            {attendedEvents.length > 0 ? (
              <div className="p-6 flex flex-col gap-5" data-testid="attended-events-list">
                {attendedEvents.map((event) => (
                  <EventCard key={event.id} {...event} />
                ))}
              </div>
            ) : (
              <div data-testid="attended-empty-state">
                <EmptyState
                  icon={<TicketIcon />}
                  heading="No events attended yet"
                  subtext="Nothing here yet. You have no public events at this time."
                />
              </div>
            )}
          </section>
        </div>
      </div>
    </div>
  );
}

export default function ProfilePage() {
  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />
      <Suspense>
        <ProfileContent />
      </Suspense>
      <Footer />
    </main>
  );
}
