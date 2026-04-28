"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import Image from "next/image";
import Link from "next/link";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import CalendarIcon from "@/public/icons/calendar.svg";
import HostingIcon from "@/public/icons/ticket-star.svg";
import PastIcon from "@/public/icons/camera-smile-01.svg";
import BubbleChatIcon from "@/public/icons/bubble-chat.svg";
import ZeroIcon from "@/public/icons/zero.svg";
import EmptyStateBg from "@/public/icons/empty-state-bg.svg";

type MyEventsTab = "upcoming" | "hosting" | "past";
type ForYouTab = "discover" | "following";

const myEventsTabs: { id: MyEventsTab; label: string; icon?: string }[] = [
  {
    id: "upcoming",
    label: "Upcoming",
    icon: CalendarIcon,
  },
  { id: "hosting", label: "Hosting", icon: HostingIcon },
  { id: "past", label: "Past", icon: PastIcon },
];

const forYouTabs: { id: ForYouTab; label: string }[] = [
  { id: "discover", label: "Discover" },
  { id: "following", label: "Following" },
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

// Mock data for My Events (Timeline)
const upcomingEvents: TimelineEvent[] = [
  {
    id: 1,
    date: "6 Mar, Friday",
    day: "Friday",
    time: "18:00 - 20:00 UTC",
    title: "Stellar Developers Meetup",
    location: "Discord",
    imageUrl: "/images/event1.png",
    isFree: true,
    attendees: 24,
    status: "going",
  },
  {
    id: 2,
    date: "8 Mar, Sunday",
    day: "Sunday",
    time: "10:00 - 12:00 UTC",
    title: "Web3 Design Workshop",
    location: "Lagos, Nigeria",
    imageUrl: "/images/event2.png",
    isFree: false,
    price: "$25.00",
    attendees: 156,
  },
  {
    id: 3,
    date: "12 Mar, Thursday",
    day: "Thursday",
    time: "14:00 - 16:00 UTC",
    title: "Blockchain Fundamentals",
    location: "Virtual",
    imageUrl: "/images/event3.png",
    isFree: true,
    attendees: 89,
  },
];

const hostingEvents: TimelineEvent[] = [
  {
    id: 4,
    date: "15 Mar, Sunday",
    day: "Sunday",
    time: "19:00 - 21:00 UTC",
    title: "Agora Community AMA",
    location: "Twitter Spaces",
    imageUrl: "/images/event4.png",
    isFree: true,
    attendees: 342,
  },
  {
    id: 5,
    date: "22 Mar, Sunday",
    day: "Sunday",
    time: "15:00 - 18:00 UTC",
    title: "NFT Ticketing Workshop",
    location: "Virtual",
    imageUrl: "/images/event5.png",
    isFree: false,
    price: "$50.00",
    attendees: 78,
  },
];

const pastEvents: TimelineEvent[] = [
  {
    id: 6,
    date: "28 Feb, Saturday",
    day: "Saturday",
    time: "16:00 - 18:00 UTC",
    title: "Crypto Trading Basics",
    location: "Discord",
    imageUrl: "/images/event6.png",
    isFree: true,
    attendees: 210,
    status: "finished",
  },
  {
    id: 7,
    date: "20 Feb, Friday",
    day: "Friday",
    time: "12:00 - 14:00 UTC",
    title: "DeFi Yield Strategies",
    location: "Virtual",
    imageUrl: "/images/event1.png",
    isFree: false,
    price: "$30.00",
    attendees: 445,
    status: "finished",
  },
];

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
          key={tab.id}
          onClick={() => onTabChange(tab.id)}
          className="relative px-3 transition-all ease-in-out sm:px-5 py-1.5 sm:py-2 text-[13px] sm:text-[15px] font-medium  duration-200 z-10  flex items-center justify-center gap-2.5 flex-row"
        >
          {activeTab === tab.id && (
            <motion.div
              layoutId={layoutId}
              className="absolute inset-0 bg-[#FFEFD3] rounded-full"
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
}: {
  title: string;
  tabs: { id: T; label: string }[];
  activeTab: T;
  onTabChange: (tab: T) => void;
  layoutId: string;
  hasNotifications?: boolean;
}) {
  return (
    <div className="flex flex-col  gap-3 sm:gap-8 mb-6 sm:mb-8">
      <h2 className="text-[24px] sm:text-[28px] lg:text-[3.6rem] leading-16.5 tracking-[0px] font-semibold text-[#131517] italic">
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
          <Link href="#">
            <div className="w-13.75 h-13.75 rounded-full bg-[#FFEFD3] flex items-center justify-center  relative">
              <div className="absolute -top-1 right-1 rounded-full size-4.75 bg-[#F90B0B] text-white flex items-center justify-center">
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
          </Link>
        )}
      </div>
    </div>
  );
}

// Timeline Event Card Component
function TimelineEventCard({ event }: { event: TimelineEvent }) {
  const locationImageSrc =
    event.location.toLowerCase().includes("discord") ||
    event.location.toLowerCase().includes("virtual") ||
    event.location.toLowerCase().includes("twitter")
      ? "/icons/discord.svg"
      : "/icons/location.svg";

  return (
    <div className="flex md:gap-22.5  ">
      {/* Timeline Column */}
      <div className="flex  w-39 max-w-39 shrink-0  mb-3">
        <span className="text-[1.625rem] text-left font-medium text-black  leading-10.25">
          {event.date}
        </span>
      </div>

      <div className="flex gap-17.5 flex-1">
        {/* divider */}

        <div className="h-full  flex flex-col gap-2">
          <div className="rounded-full size-4.25 bg-black opacity-50" />
          <div className="h-full w-0 border-[1.5px] border-dashed border-black  mx-auto flex-1 relative">
            <div className="absolute w-1 h-full -left-0.5  bg-linear-to-b from-transparent to-[#FFFBE9] z-20" />
          </div>
        </div>
        {/* Event Card */}
        <Link href={`/events/${event.id}`} className="   h-full flex-1">
          <div className="bg-[#FFEFD3] rounded-xl  shadow-[-4px_4px_0_rgba(0,0,0,1)] sm:shadow-[-6px_6px_0_rgba(0,0,0,1)] p-9.5 overflow-hidden transition-all ease-in-out hover:-translate-x-0.5 hover:translate-y-0.5 hover:shadow-[-3px_3px_0_rgba(0,0,0,1)] sm:hover:-translate-x-1 sm:hover:translate-y-1 sm:hover:shadow-[-4px_4px_0_rgba(0,0,0,1)]">
            <div className="flex flex-col sm:flex-row gap-6">
              {/* Left side - Image */}
              <div className="w-full flex-1 ">
                <Image
                  src={event.imageUrl}
                  width={400}
                  height={140}
                  alt={event.title}
                  className="object-cover w-full h-full"
                />
              </div>

              {/* Right side - Details */}
              <div className="flex-1 p-3 sm:p-4 flex flex-col min-w-0">
                <div className="flex items-start justify-between gap-2">
                  <div className="min-w-0 flex-1">
                    <p className="text-[15px] text-black font-light leading-7.5 tracking-[0%] mb-4.5">
                      {event.time}
                    </p>
                    <h3 className="text-[1.2rem] font-semibold text-black leading-5.5 line-clamp-2 mb-4.5">
                      {event.title}
                    </h3>
                  </div>
                  {/* <span className="text-[12px] sm:text-[14px] font-semibold text-black shrink-0">
                    {event.isFree ? "Free" : event.price}
                  </span> */}
                </div>

                <div className="">
                  <div className="flex items-center gap-1.5 text-black/70">
                    <Image
                      src={locationImageSrc}
                      alt="location"
                      width={16}
                      height={16}
                      className="object-contain"
                    />
                    <span className="text-[12px] text-black ">
                      {event.location}
                    </span>
                  </div>

                  <div className="flex items-center justify-between mt-2 sm:mt-3">
                    <div className="flex items-center gap-1.5 sm:gap-2">
                      {/* barge */}
                      {event.status && (
                        <div
                          className={`capitalize rounded-lg p-2.5 ${event.status === "going" ? "bg-[#DAFFB5] text-black" : event.status === "finished" ? "bg-[#FFFBE9] text-black" : ""} w-20.5 text-center text-xs font-medium`}
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
                            {
                              <Image
                                src="/images/pfp.png"
                                width={24}
                                height={24}
                                alt="attendee"
                                className="object-cover w-full h-full"
                              />
                            }
                          </div>
                        ))}
                      </div>
                      <span className="text-[11px] sm:text-[12px] text-black/60">
                        {event.attendees} going
                      </span>
                    </div>

                    <div className="flex items-center gap-1 text-black text-[12px] sm:text-[13px] font-medium">
                      <span className="hidden sm:inline">View Event</span>
                      <span className="sm:hidden">View</span>
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
function GridEventCard({ event }: { event: GridEvent }) {
  return (
    <Link href={`/events/${event.id}`} className="block">
      <div
        className={`${event.color} rounded-xl border border-black shadow-[-4px_4px_0_rgba(0,0,0,1)] sm:shadow-[-6px_6px_0_rgba(0,0,0,1)] overflow-hidden transition-transform hover:-translate-x-0.5 hover:translate-y-0.5 hover:shadow-[-3px_3px_0_rgba(0,0,0,1)] sm:hover:-translate-x-1 sm:hover:translate-y-1 sm:hover:shadow-[-4px_4px_0_rgba(0,0,0,1)]`}
      >
        {/* Image */}
        <div className="h-[120px] sm:h-[140px] overflow-hidden">
          <Image
            src={event.imageUrl}
            width={400}
            height={140}
            alt={event.title}
            className="object-cover w-full h-full"
          />
        </div>

        {/* Content */}
        <div className="p-3 sm:p-4">
          <h3 className="text-[13px] sm:text-[14px] font-semibold text-black leading-tight mb-1.5 sm:mb-2 line-clamp-2">
            {event.title}
          </h3>

          <p className="text-[11px] sm:text-[12px] text-black/60 mb-1">
            {event.date}
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
              {event.location}
            </span>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-[12px] sm:text-[13px] font-medium text-black">
              {event.price === "$0.00" ? "Free" : event.price}
            </span>
            <div className="flex items-center gap-1 text-black text-[11px] sm:text-[12px] font-medium">
              <span className="hidden sm:inline">View</span>
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

// My Events Section Content
function MyEventsContent({ activeTab }: { activeTab: MyEventsTab }) {
  let events: TimelineEvent[] = [];

  switch (activeTab) {
    case "upcoming":
      events = upcomingEvents;
      break;
    case "hosting":
      events = hostingEvents;
      break;
    case "past":
      events = pastEvents;
      break;
  }

  if (events.length === 0) {
    return (
      <div className="w-full max-w-121.5 bg-[#FFEFD3] h-107.5 rounded-4xl mx-auto flex flex-col items-center justify-center gap-10 text-[#1315175C]">
        <div className="max-w-56 w-full bg-white rounded-4xl h-56 relative p-5.5">
          <Image
            src={EmptyStateBg}
            alt="Empty State Background"
            width={224}
            height={224}
            className="object-cover w-full h-full rounded-4xl"
          />

          <div className="bg-white absolute max-w-23.75 rounded-4xl max-h-23.75 w-full h-full shadow-[#63636312] -top-7 -right-7 shadow-[0px_1.65px_1.32px_0px] flex items-center justify-center p-3">
            <Image
              src={ZeroIcon}
              alt="Nothing Here, Yet"
              width={16}
              height={16}
              className="object-center w-full h-full"
            />
          </div>
        </div>
        <div className="flex flex-col items-center gap-4">
          <p className="text-xl font-medium leading-5.5 text-center">Nothing Here, Yet</p>
          <Link href="/create-event">
            <button className="bg-black text-white px-6 py-2 rounded-full font-medium shadow-[-4px_4px_0_rgba(0,0,0,0.2)] hover:shadow-none hover:translate-x-[-2px] hover:translate-y-[2px] transition-all">
              Create Your First Event
            </button>
          </Link>
        </div>
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
function ForYouContent({ activeTab }: { activeTab: ForYouTab }) {
  let events: GridEvent[] = [];

  switch (activeTab) {
    case "discover":
      events = discoverEvents;
      break;
    case "following":
      events = followingEvents;
      break;
  }

  if (events.length === 0) {
    return (
      <div className="min-h-[200px] rounded-xl border-2 border-dashed border-black/20 flex items-center justify-center">
        <p className="text-black/50 text-lg">No events found</p>
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

export default function HomePage() {
  const [myEventsTab, setMyEventsTab] = useState<MyEventsTab>("upcoming");
  const [forYouTab, setForYouTab] = useState<ForYouTab>("discover");

  return (
    <div className="min-h-screen bg-[#FFFDF0]">
      <Navbar />

      <main className="w-full max-w-304.5 mx-auto px-3 sm:px-4 lg:px-6 xl:px-0 pt-6 sm:pt-22.5 pb-12 sm:pb-20">
        {/* My Events Section */}
        <section className="mb-10 sm:mb-16 space-y-15">
          <SectionHeader
            title="My Events"
            tabs={myEventsTabs}
            activeTab={myEventsTab}
            onTabChange={setMyEventsTab}
            layoutId="my-events-toggle"
            hasNotifications={true}
          />
          <MyEventsContent activeTab={myEventsTab} />
        </section>

        {/* For You Section */}
        <section>
          <SectionHeader
            title="For You"
            tabs={forYouTabs}
            activeTab={forYouTab}
            onTabChange={setForYouTab}
            layoutId="for-you-toggle"
          />
          <ForYouContent activeTab={forYouTab} />
        </section>
      </main>

      <Footer />
    </div>
  );
}
