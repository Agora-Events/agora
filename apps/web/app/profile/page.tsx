"use client";

import { Suspense, useState, useEffect } from "react";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { ProfileSidebar } from "@/components/profile/profile-sidebar";
import { EventCard } from "@/components/events/event-card";
import Image from "next/image";
import Link from "next/link";
import { useSearchParams } from "next/navigation";

// Types for organizer profile
interface OrganizerProfile {
  id?: string;
  address: string;
  displayName: string;
  bio?: string;
  avatarUrl?: string;
  socials?: Record<string, string>;
  createdAt?: string;
  updatedAt?: string;
}

// Types for events
interface EventItem {
  id: number;
  title: string;
  date: string;
  location: string;
  price: string;
  imageUrl: string;
}

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

const CalendarIcon = () => (
  <Image src="/icons/calendar.svg" width={32} height={32} alt="Calendar" className="text-amber-400" />
);

const TicketIcon = () => (
  <Image src="/icons/ticket.svg" width={32} height={32} alt="Ticket" className="text-amber-400" />
);

// New component to display organizer profile information
function OrganizerProfileSection({ profile }: { profile: OrganizerProfile | null }) {
  if (!profile) {
    return (
      <div className="bg-white rounded-2xl border border-border-warm shadow-sm p-6">
        <h2 className="text-lg font-semibold text-ink-soft mb-4">Organizer Profile</h2>
        <p className="text-gray-500">No profile information available</p>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-2xl border border-border-warm shadow-sm p-6">
      <h2 className="text-lg font-semibold text-ink-soft mb-4">Organizer Profile</h2>
      <div className="flex items-start gap-4">
        {profile.avatarUrl ? (
          <Image
            src={profile.avatarUrl}
            alt={profile.displayName}
            width={80}
            height={80}
            className="rounded-full object-cover"
          />
        ) : (
          <div className="w-20 h-20 rounded-full bg-violet-100 flex items-center justify-center text-2xl font-bold text-violet-600">
            {profile.displayName.charAt(0).toUpperCase()}
          </div>
        )}
        <div className="flex-1">
          <h3 className="text-xl font-bold text-ink-deep">{profile.displayName}</h3>
          {profile.bio && <p className="text-gray-600 mt-2">{profile.bio}</p>}
          <div className="mt-4 flex flex-wrap gap-2">
            {profile.socials && Object.entries(profile.socials).map(([platform, url]) => (
              <a
                key={platform}
                href={url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm bg-violet-50 text-violet-700 px-3 py-1 rounded-full hover:bg-violet-100 transition-colors"
              >
                {platform}
              </a>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

function ProfileContent() {
  const searchParams = useSearchParams();
  const isEmpty = searchParams.get("empty") === "1";
  const profileAddress = searchParams.get("address") ?? "me";
  const [profile, setProfile] = useState<OrganizerProfile | null>(null);
  const [_loading, setLoading] = useState(true);
  const [_error, setError] = useState<string | null>(null);

  const hostedEvents = isEmpty ? [] : HOSTED_EVENTS;
  const attendedEvents = isEmpty ? [] : ATTENDED_EVENTS;

  useEffect(() => {
    const fetchProfile = async () => {
      try {
        setLoading(true);
        const response = await fetch("/api/profile");
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data = await response.json();
        setProfile(data.profile);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load profile");
      } finally {
        setLoading(false);
      }
    };

    fetchProfile();
  }, []);

  return (
    <div className="flex-1 w-full max-w-6xl mx-auto px-4 py-10">
      <div className="flex flex-col md:flex-row gap-8 items-start">
        <div className="w-full md:w-[28%] md:sticky md:top-24">
          <ProfileSidebar address={profileAddress} />
        </div>

        <div className="flex-1 flex flex-col gap-6">
          {/* Organizer Profile Section */}
          <OrganizerProfileSection profile={profile} />

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
