"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import useSWR from "swr";
import { motion } from "framer-motion";
import Image from "next/image";
import Link from "next/link";
import { useTranslations } from "next-intl";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { ChatSidebar } from "@/components/layout/chat-sidebar";
import { useAuth } from "@/hooks/useAuth";
import { UpcomingEventsEmptyState } from "@/components/empty-state/upcoming-events-empty-state";
import CalendarIcon from "@/public/icons/calendar.svg";
import HostingIcon from "@/public/icons/ticket-star.svg";
import PastIcon from "@/public/icons/camera-smile-01.svg";
import BubbleChatIcon from "@/public/icons/bubble-chat.svg";

type MyEventsTab = "upcoming" | "hosting" | "past";
type ForYouTab = "discover" | "following";

const myEventsTabs = (t: (key: string) => string): { id: MyEventsTab; label: string; icon?: string }[] => [
  {
    id: "upcoming",
    label: t("tabUpcoming"),
    icon: CalendarIcon,
  },
  { id: "hosting", label: t("tabHosting"), icon: HostingIcon },
  { id: "past", label: t("tabPast"), icon: PastIcon },
];

const forYouTabs = (t: (key: string) => string): { id: ForYouTab; label: string }[] => [
  { id: "discover", label: t("tabDiscover") },
  { id: "following", label: t("tabFollowing") },
];

// Mock data types
interface TimelineEvent {
  id: number;
  date: string;
  day: string;
  time: string;
  title: string;
  location: string;
  imageUrl: string;
  isFree: boolean;
  price?: string;
  attendees: number;
  status?: string;
}

interface GridEvent {
  id: number;
  title: string;
  date: string;
  location: string;
  price: string;
  imageUrl: string;
  color: string;
}

// Mock data for For You (Grid)
const discoverEvents: GridEvent[] = [
  {
    id: 8,
    title: "Stellar Consensus Protocol",
    date: "Apr 15, 2026",
    location: "Austin, TX",
    price: "$0.00",
    imageUrl: "/images/event2.png",
    color: "bg-[#E8D5F7]",
  },
  {
    id: 9,
    title: "Real Estate Outlook 2026",
    date: "Apr 20, 2026",
    location: "New York, NY",
    price: "$45.00",
    imageUrl: "/images/event3.png",
    color: "bg-[#F7D5D5]",
  },
  {
    id: 10,
    title: "Web3 Marketing Summit",
    date: "May 5, 2026",
    location: "London, UK",
    price: "$0.00",
    imageUrl: "/images/event4.png",
    color: "bg-[#D5F7E8]",
  },
  {
    id: 11,
    title: "AI & Blockchain Convergence",
    date: "May 12, 2026",
    location: "San Francisco, CA",
    price: "$75.00",
    imageUrl: "/images/event5.png",
    color: "bg-[#F7ECD5]",
  },
  {
    id: 12,
    title: "Developer Workshop Series",
    date: "May 18, 2026",
    location: "Virtual",
    price: "$0.00",
    imageUrl: "/images/event6.png",
    color: "bg-[#D5E8F7]",
  },
  {
    id: 13,
    title: "Crypto Investment Forum",
    date: "Jun 2, 2026",
    location: "Singapore",
    price: "$120.00",
    imageUrl: "/images/event1.png",
    color: "bg-[#F5D5F7]",
  },
];

const followingEvents: GridEvent[] = [
  {
    id: 14,
    title: "Stellar East Africa Meetup",
    date: "Apr 10, 2026",
    location: "Nairobi, Kenya",
    price: "$0.00",
    imageUrl: "/images/event3.png",
    color: "bg-[#F7D5E8]",
  },
  {
    id: 15,
    title: "Women in Web3 Panel",
    date: "Apr 25, 2026",
    location: "Virtual",
    price: "$0.00",
    imageUrl: "/images/event2.png",
    color: "bg-[#E8F7D5]",
  },
  {
    id: 16,
    title: "Smart Contract Security",
    date: "May 8, 2026",
    location: "Berlin, Germany",
    price: "$35.00",
    imageUrl: "/images/event5.png",
    color: "bg-[#D5F5F7]",
  },
  {
    id: 17,
    title: "Community Builder Workshop",
    date: "May 20, 2026",
    location: "Toronto, Canada",
    price: "$0.00",
    imageUrl: "/images/event4.png",
    color: "bg-[#F7E8D5]",
  },
];

function AnimatedToggle<T extends string>({
  tabs,
  activeTab,
  onTabChange,
  layoutId,
}: {
  tabs: { id: T; label: string; icon?: string }[];
  activeTab: T;
  onTabChange: (tab: T) => void;
  layoutId: string;
}) {
  return (
    <div className="inline-flex w-fit items-center bg-white rounded-full p-1 sm:p-1.5 ">
      {tabs.map((tab) => (
        <button
          type="button"
          key={tab.id}
          onClick={() => onTabChange(tab.id)}
          className="relative px-3 transition-all ease-in-out sm:px-5 py-1.5 sm:py-2 text-[13px] sm:text-[15px] font-medium  duration-200 z-10  flex items-center justify-center gap-2.5 flex-row"
        >
          {activeTab === tab.id && (
            <motion.div
              layoutId={layoutId}
              className="absolute inset-0 bg-surface rounded-full"
              transition={{
                type: "spring",
                stiffness: 400,
                damping: 30,
              }}
            />
          )}
          {tab.icon && (
            <Image
              src={tab.icon}
              alt={`${tab.label} icon`}
              width={16}
              height={16}
              className="object-contain w-4 h-4 sm:w-6 sm:h-6 relative"
            />
          )}

          <span
            className={`relative z-10 text-sm leading-7.5 tracking-[0%] ${
              activeTab === tab.id ? "text-black font-bold" : "text-black/70"
            }`}
          >
            {tab.label}
          </span>
        </button>
      ))}
    </div>
  );
}

function SectionHeader<T extends string>({
  title,
  tabs,
  activeTab,
  onTabChange,
  layoutId,
  hasNotifications = false,
  onChatClick,
}: {
  title: string;
  tabs: { id: T; label: string }[];
  activeTab: T;
  onTabChange: (tab: T) => void;
  layoutId: string;
  hasNotifications?: boolean;
  onChatClick?: () => void;
}) {
  const t = useTranslations("home");
  return (
    <div className="flex flex-col  gap-3 sm:gap-8 mb-6 sm:mb-8">
      <h2 className="text-[24px] sm:text-[28px] lg:text-[3.6rem] leading-16.5 tracking-[0px] font-semibold text-ink-deep italic">
        {title}
      </h2>
      <div className="flex justify-between items-end">
        <AnimatedToggle
          tabs={tabs}
          activeTab={activeTab}
          onTabChange={onTabChange}
          layoutId={layoutId}
        />
        {hasNotifications && (
          <button type="button" onClick={onChatClick} aria-label={t("openMessages")}>
            <div className="w-13.75 h-13.75 rounded-full bg-surface flex items-center justify-center  relative">
              <div className="absolute -top-1 right-1 rounded-full size-4.75 bg-error text-white flex items-center justify-center">
                <p>1</p>
              </div>
              <Image
                src={BubbleChatIcon}
                alt="chat"
                width={24}
                height={24}
                className="object-contain w-6 h-6 mx-auto"
              />
            </div>
          </button>
        )}
      </div>
    </div>
  );
}

// Timeline Event Card Component
function TimelineEventCard({ event }: { event: any }) {
  const t = useTranslations("home");
  const locationImageSrc =
    (event.location || "").toLowerCase().includes("discord") ||
    (event.location || "").toLowerCase().includes("virtual") ||
    (event.location || "").toLowerCase().includes("twitter")
      ? "/icons/discord.svg"
      : "/icons/location.svg";

  return (
    <div className="flex md:gap-22.5  ">
      {/* Timeline Column */}
      <div className="flex  w-39 max-w-39 shrink-0  mb-3">
        <span className="text-[1.625rem] text-left font-medium text-black  leading-10.25">
          {event.date || t("tbd")}
        </span>
      </div>

      <div className="flex gap-17.5 flex-1">
        {/* divider */}

        <div className="h-full  flex flex-col gap-2">
          <div className="rounded-full size-4.25 bg-black opacity-50" />
          <div className="h-full w-0 border-[1.5px] border-dashed border-black  mx-auto flex-1 relative">
            <div className="absolute w-1 h-full -left-0.5  bg-linear-to-b from-transparent to-base z-20" />
          </div>
        </div>
        {/* Event Card */}
        <Link href={`/events/${event.id}`} className="   h-full flex-1">
          <div className="bg-surface rounded-xl  shadow-[-4px_4px_0_rgba(0,0,0,1)] sm:shadow-[-6px_6px_0_rgba(0,0,0,1)] p-9.5 overflow-hidden transition-all ease-in-out hover:-translate-x-0.5 hover:translate-y-0.5 hover:shadow-[-3px_3px_0_rgba(0,0,0,1)] sm:hover:-translate-x-1 sm:hover:translate-y-1 sm:hover:shadow-[-4px_4px_0_rgba(0,0,0,1)]">
            <div className="flex flex-col sm:flex-row gap-6">
              {/* Left side - Image */}
              <div className="w-full flex-1 ">
                <Image
                  src={event.imageUrl || "/images/event1.png"}
                  width={400}
                  height={140}
                  alt={event.title || "Event"}
                  className="object-cover w-full h-full"
                />
              </div>

              {/* Right side - Details */}
              <div className="flex-1 p-3 sm:p-4 flex flex-col min-w-0">
                <div className="flex items-start justify-between gap-2">
                  <div className="min-w-0 flex-1">
                    <p className="text-[15px] text-black font-light leading-7.5 tracking-[0%] mb-4.5">
                      {event.time || "12:00 UTC"}
                    </p>
                    <h3 className="text-[1.2rem] font-semibold text-black leading-5.5 line-clamp-2 mb-4.5">
                      {event.title}
                    </h3>
                  </div>
                </div>

                <div className="">
                  <div className="flex items-center gap-1.5 text-black/70">
                    <Image
                      src={locationImageSrc}
                      alt="Location"
                      width={16}
                      height={16}
                      className="object-contain"
                    />
                    <span className="text-[12px] text-black ">
                      {event.location || t("virtual")}
                    </span>
                  </div>

                  <div className="flex items-center justify-between mt-2 sm:mt-3">
                    <div className="flex items-center gap-1.5 sm:gap-2">
                      {event.status && (
                        <div
                          className={`capitalize rounded-lg p-2.5 ${event.status === "going" ? "bg-success-light text-black" : event.status === "finished" ? "bg-base text-black" : ""} w-20.5 text-center text-xs font-medium`}
                        >
                          {event.status}
                        </div>
                      )}
                      <div className="flex -space-x-1.5 sm:-space-x-2">
                        {[1, 2, 3].map((i) => (
                          <div
                            key={i}
                            className="w-5 h-5 sm:w-6 sm:h-6 rounded-full border-2 border-white overflow-hidden bg-gray-300"
                          >
                            <Image
                              src="/images/pfp.png"
                              width={24}
                              height={24}
                              alt="attendee"
                              className="object-cover w-full h-full"
                            />
                          </div>
                        ))}
                      </div>
                      <span className="text-[11px] sm:text-[12px] text-black/60">
                        {event.attendees || 0} {t("going")}
                      </span>
                    </div>

                    <div className="flex items-center gap-1 text-black text-[12px] sm:text-[13px] font-medium">
                      <span className="hidden sm:inline">{t("viewEvent")}</span>
                      <span className="sm:hidden">{t("view")}</span>
                      <Image
                        src="/icons/arrow-right.svg"
                        width={16}
                        height={16}
                        alt="arrow"
                        className="object-contain w-4 h-4 sm:w-[18px] sm:h-[18px]"
                      />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </Link>
      </div>
    </div>
  );
}

// Grid Event Card Component
function GridEventCard({ event }: { event: any }) {
  const t = useTranslations("home");
  const color = event.color || "bg-[#E8D5F7]";
  return (
    <Link href={`/events/${event.id}`} className="block">
      <div
        className={`${color} rounded-xl border border-black shadow-[-4px_4px_0_rgba(0,0,0,1)] sm:shadow-[-6px_6px_0_rgba(0,0,0,1)] overflow-hidden transition-transform hover:-translate-x-0.5 hover:translate-y-0.5 hover:shadow-[-3px_3px_0_rgba(0,0,0,1)] sm:hover:-translate-x-1 sm:hover:translate-y-1 sm:hover:shadow-[-4px_4px_0_rgba(0,0,0,1)]`}
      >
        {/* Image */}
        <div className="h-[120px] sm:h-[140px] overflow-hidden">
          <Image
            src={event.imageUrl || "/images/event2.png"}
            width={400}
            height={140}
            alt={event.title || "Event"}
            className="object-cover w-full h-full"
          />
        </div>

        {/* Content */}
        <div className="p-3 sm:p-4">
          <h3 className="text-[13px] sm:text-[14px] font-semibold text-black leading-tight mb-1.5 sm:mb-2 line-clamp-2">
            {event.title}
          </h3>

          <p className="text-[11px] sm:text-[12px] text-black/60 mb-1">
            {event.date || t("tbd")}
          </p>

          <div className="flex items-center gap-1 text-black/70 mb-2 sm:mb-3">
            <Image
              src="/icons/location.svg"
              alt="location"
              width={12}
              height={12}
              className="object-contain w-3 h-3 sm:w-[14px] sm:h-[14px]"
            />
            <span className="text-[11px] sm:text-[12px] line-clamp-1">
              {event.location || t("virtual")}
            </span>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-[12px] sm:text-[13px] font-medium text-black">
              {event.price === "$0.00" || !event.price ? t("free") : event.price}
            </span>
            <div className="flex items-center gap-1 text-black text-[11px] sm:text-[12px] font-medium">
              <span className="hidden sm:inline">{t("view")}</span>
              <Image
                src="/icons/arrow-right.svg"
                width={14}
                height={14}
                alt="arrow"
                className="object-contain w-3.5 h-3.5 sm:w-4 sm:h-4"
              />
            </div>
          </div>
        </div>
      </div>
    </Link>
  );
}

function EventCardSkeleton() {
  return (
    <div className="min-h-[200px] rounded-xl border-2 border-black/20 bg-black/5 animate-pulse flex items-center justify-center">
    </div>
  );
}

// My Events Section Content
function MyEventsContent({
  activeTab,
  events,
  isLoading,
}: {
  activeTab: MyEventsTab;
  events: any[];
  isLoading: boolean;
}) {
  const t = useTranslations("home");
  const isUpcomingTab = activeTab === "upcoming";

  if (isLoading) {
    return (
      <div className="pt-4 space-y-13.25">
        <EventCardSkeleton />
        <EventCardSkeleton />
      </div>
    );
  }

  if (events.length === 0) {
    if (isUpcomingTab) {
      return <UpcomingEventsEmptyState />;
    }

    return (
      <div className="flex min-h-[15rem] items-center justify-center rounded-[2rem] border border-dashed border-black/20 bg-white/60 px-6 text-center">
        <p className="text-base font-medium text-black/55">
          {t("noEventsFound")}
        </p>
      </div>
    );
  }

  return (
    <div className="pt-4 space-y-13.25">
      {events.map((event) => (
        <TimelineEventCard key={event.id} event={event} />
      ))}
    </div>
  );
}

// For You Section Content
function ForYouContent({ activeTab, events, isLoading }: { activeTab: ForYouTab, events: any[], isLoading: boolean }) {
  const t = useTranslations("home");
  if (isLoading) {
    return (
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-5 lg:gap-6">
        <EventCardSkeleton />
        <EventCardSkeleton />
        <EventCardSkeleton />
      </div>
    );
  }

  if (events.length === 0) {
    return (
      <div className="min-h-[200px] rounded-xl border-2 border-dashed border-black/20 flex items-center justify-center">
        <p className="text-black/50 text-lg">{t("noEventsFound")}</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-5 lg:gap-6">
      {events.map((event) => (
        <GridEventCard key={event.id} event={event} />
      ))}
    </div>
  );
}

const fetcher = (url: string) => fetch(url).then((res) => res.json());

export default function HomePage() {
  const router = useRouter();
  const t = useTranslations("home");
  const {
    walletAddress: userWallet,
    isAuthenticated,
    isLoading: isAuthLoading,
  } = useAuth();
  const [myEventsTab, setMyEventsTab] = useState<MyEventsTab>("upcoming");
  const [forYouTab, setForYouTab] = useState<ForYouTab>("discover");
  const [isChatOpen, setIsChatOpen] = useState(false);

  // "My Events" is personal — signed-out visitors belong on the auth page.
  useEffect(() => {
    if (!isAuthLoading && !isAuthenticated) {
      router.replace("/auth");
    }
  }, [isAuthLoading, isAuthenticated, router]);

  const { data, isLoading: isEventsLoading } = useSWR(
    isAuthenticated ? "/api/v1/events" : null,
    fetcher,
  );
  const isLoading = isAuthLoading || isEventsLoading;
  const allEvents = data?.events || [];

  const now = new Date().getTime();

  // Filter for 'My Events'
  const upcomingEvents = allEvents.filter((e: any) => new Date(e.start_time).getTime() > now); // assuming user has ticket logically mapped
  const hostingEvents = userWallet
    ? allEvents.filter((e: any) => e.organizer_wallet === userWallet)
    : [];
  const pastEvents = allEvents.filter((e: any) => new Date(e.end_time).getTime() < now);

  let displayedMyEvents = [];
  if (myEventsTab === "upcoming") displayedMyEvents = upcomingEvents;
  else if (myEventsTab === "hosting") displayedMyEvents = hostingEvents;
  else if (myEventsTab === "past") displayedMyEvents = pastEvents;

  // Filter for 'For You'
  const discoverEvents = allEvents.slice(0, 6); // simple pagination mock
  const followingEvents = allEvents.slice(0, 4);

  let displayedForYouEvents = [];
  if (forYouTab === "discover") displayedForYouEvents = discoverEvents;
  else if (forYouTab === "following") displayedForYouEvents = followingEvents;

  return (
    <div className="min-h-screen bg-base-alt">
      <Navbar />

      <main className="w-full max-w-304.5 mx-auto px-3 sm:px-4 lg:px-6 xl:px-0 pt-6 sm:pt-22.5 pb-12 sm:pb-20">
        {/* My Events Section */}
        <section className="mb-10 sm:mb-16 space-y-15">
          <SectionHeader
            title={t("myEvents")}
            tabs={myEventsTabs(t)}
            activeTab={myEventsTab}
            onTabChange={setMyEventsTab}
            layoutId="my-events-toggle"
            hasNotifications={true}
            onChatClick={() => setIsChatOpen((prev) => !prev)}
          />

          {/* Chat Sidebar (shown when toggled) */}
          {isChatOpen && (
            <div className="flex justify-end mb-4">
              <ChatSidebar onNewChat={() => setIsChatOpen(false)} />
            </div>
          )}

          <MyEventsContent activeTab={myEventsTab} events={displayedMyEvents} isLoading={isLoading} />
        </section>

        {/* For You Section */}
        <section>
          <SectionHeader
            title={t("forYou")}
            tabs={forYouTabs(t)}
            activeTab={forYouTab}
            onTabChange={setForYouTab}
            layoutId="for-you-toggle"
          />
          <ForYouContent activeTab={forYouTab} events={displayedForYouEvents} isLoading={isLoading} />
        </section>
      </main>

      <Footer />
    </div>
  );
}

