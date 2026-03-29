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
      <div className="w-20 h-20 rounded-full bg-[#FFEFD3] flex items-center justify-center mb-5">
        {icon}
      </div>
      <h3 className="text-[#1A1A1A] font-semibold text-lg mb-2">{heading}</h3>
      <p className="text-gray-500 text-sm max-w-xs mb-6">{subtext}</p>
      <Link
        href="/events"
        className="inline-flex items-center gap-2 bg-[#1A1A1A] text-white text-sm font-medium px-5 py-2.5 rounded-full hover:bg-[#333] transition-colors"
      >
        Explore Events
      </Link>
    </div>
  );
}

const CalendarIcon = () => (
  <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#C9A84C" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
    <rect x="3" y="4" width="18" height="18" rx="2" />
    <line x1="16" y1="2" x2="16" y2="6" />
    <line x1="8" y1="2" x2="8" y2="6" />
    <line x1="3" y1="10" x2="21" y2="10" />
  </svg>
);

const TicketIcon = () => (
  <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#C9A84C" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
    <path d="M2 9a3 3 0 0 1 0 6v2a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-2a3 3 0 0 1 0-6V7a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2v2z" />
    <line x1="9" y1="12" x2="15" y2="12" />
  </svg>
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
          <section className="bg-white rounded-2xl border border-[#F0EAD6] shadow-sm overflow-hidden">
            <div className="px-6 pt-6 pb-4 border-b border-[#F0EAD6]">
              <h2 className="text-lg font-semibold text-[#1A1A1A]">Hosting</h2>
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
          <section className="bg-white rounded-2xl border border-[#F0EAD6] shadow-sm overflow-hidden">
            <div className="px-6 pt-6 pb-4 border-b border-[#F0EAD6]">
              <h2 className="text-lg font-semibold text-[#1A1A1A]">Events</h2>
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
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />
      <Suspense>
        <ProfileContent />
      </Suspense>
      <Footer />
    </main>
  );
}
