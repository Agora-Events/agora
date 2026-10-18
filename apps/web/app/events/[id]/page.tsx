import type { Metadata } from "next";
import Image from "next/image";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { RegistrationBox } from "@/components/events/registration-box";
import { ReportEventLink } from "@/components/events/ReportEventLink";
import { notFound } from "next/navigation";
import MapClient from "@/components/events/map-client";
import { buildMetadata } from "@/components/layout/seo";
import { Breadcrumb } from "@/components/ui/breadcrumb";
import { EventPageView } from "@/components/analytics/event-page-view";
import { SecondaryMarketplaceTab } from "@/components/events/secondary-marketplace-tab";
import { EventTimeDisplay } from "@/components/events/event-time-display";
import ShareButton from "@/components/events/ShareButton";
import { T } from "@/components/ui/t";
import { AddToCalendar } from "@/components/events/add-to-calendar";

const SITE_URL = process.env.NEXT_PUBLIC_SITE_URL ?? "https://agora.events";
const BACKEND_URL =
  process.env.BACKEND_URL ??
  process.env.NEXT_PUBLIC_API_URL ??
  "http://localhost:3001";

interface EventPageData {
  id: string;
  title: string;
  description: string;
  location: string;
  startsAt: string;
  date: string;
  price: string;
  imageUrl: string;
  category: string;
  host: {
    name: string;
    avatar: string;
    handle: string;
    hostPfp: string;
  };
}

interface ApiEventDetail {
  id: string;
  title: string;
  description: string | null;
  location: string;
  start_time: string;
  image_url: string | null;
  is_free: boolean;
  organizer_profile?: {
    address: string;
    display_name: string;
    avatar_url: string | null;
  } | null;
  tiers?: Array<{ price: string | number }> | null;
}

async function fetchEvent(id: string): Promise<EventPageData | null> {
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(id)) {
    return null;
  }

  const response = await fetch(
    `${BACKEND_URL}/api/v1/events/${encodeURIComponent(id)}?include_tiers=true`,
    { next: { revalidate: 60 } },
  );
  if (response.status === 404) return null;
  if (!response.ok) {
    throw new Error(`Event lookup failed (${response.status}).`);
  }

  const result = (await response.json()) as { data?: ApiEventDetail };
  const event = result.data;
  if (!event?.id || !event.title || !event.start_time) return null;

  const startDate = new Date(event.start_time);
  const date = Number.isNaN(startDate.getTime())
    ? "Date to be announced"
    : new Intl.DateTimeFormat("en-US", {
        weekday: "short",
        month: "short",
        day: "numeric",
        hour: "numeric",
        minute: "2-digit",
        timeZoneName: "short",
      }).format(startDate);
  const tierPrices = (event.tiers ?? [])
    .map((tier) => Number(tier.price))
    .filter((price) => Number.isFinite(price));
  const startingPrice = tierPrices.length ? Math.min(...tierPrices) : null;
  const price =
    event.is_free || startingPrice === 0
      ? "Free"
      : startingPrice === null
        ? "Free"
        : startingPrice.toFixed(2);
  const organizer = event.organizer_profile;
  const wallet = organizer?.address ?? "";
  const abbreviatedWallet = wallet
    ? `${wallet.slice(0, 5)}…${wallet.slice(-4)}`
    : "Event organizer";

  return {
    id: event.id,
    title: event.title,
    description: event.description ?? "",
    location: event.location,
    startsAt: event.start_time,
    date,
    price,
    imageUrl: event.image_url || "/images/event1.png",
    category: "Event",
    host: {
      name: organizer?.display_name || "Event organizer",
      avatar: organizer?.avatar_url?.startsWith("/")
        ? organizer.avatar_url
        : "/icons/stellar-logo.svg",
      handle: abbreviatedWallet,
      hostPfp: organizer?.avatar_url?.startsWith("/")
        ? organizer.avatar_url
        : "/images/pfp.png",
    },
  };
}

function truncateDescription(text: string, maxLength = 160): string {
  if (text.length <= maxLength) return text;
  return `${text.slice(0, maxLength - 1).trimEnd()}…`;
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ id: string }>;
}): Promise<Metadata> {
  const { id } = await params;
  const event = await fetchEvent(id);
  if (!event) return { title: "Event not found" };
  const ogImageUrl = `${SITE_URL}/api/og?eventId=${id}`;
  return buildMetadata({
    title: event.title,
    description: truncateDescription(
      `Join us for ${event.title} on ${event.date} in ${event.location}. ${event.price === "Free" ? "Free entry." : `Tickets from $${event.price}.`} Secure your spot on Agora.`
    ),
    image: ogImageUrl,
    path: `/events/${id}`,
  });
}

function truncateTitle(title: string, maxLength = 40): string {
  if (title.length <= maxLength) return title;
  return `${title.slice(0, maxLength - 1).trimEnd()}…`;
}

export default async function EventDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const event = await fetchEvent(id);

  if (!event) {
    notFound();
  }

  const host = event.host;

  const breadcrumbJsonLd = {
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: [
      { "@type": "ListItem", position: 1, name: "Home", item: SITE_URL },
      { "@type": "ListItem", position: 2, name: "Discover", item: `${SITE_URL}/discover` },
      { "@type": "ListItem", position: 3, name: event.title, item: `${SITE_URL}/events/${id}` },
    ],
  };

  const isOnline = event.location === "Online";
  const eventUrl = `${SITE_URL}/events/${id}`;
  const eventDescription = event.description || `Join us for ${event.title} on ${event.date} in ${event.location}.`;
  const parsedStartDate = new Date(event.startsAt);
  const eventImage = event.imageUrl.startsWith("http")
    ? event.imageUrl
    : `${SITE_URL}${event.imageUrl}`;

  const eventJsonLd = {
    "@context": "https://schema.org",
    "@type": "Event",
    name: event.title,
    ...(isNaN(parsedStartDate.getTime())
      ? {}
      : { startDate: parsedStartDate.toISOString() }),
    location: isOnline
      ? { "@type": "VirtualLocation", url: eventUrl }
      : { "@type": "Place", name: event.location },
    image: eventImage,
    description: eventDescription,
    eventAttendanceMode: isOnline
      ? "https://schema.org/OnlineEventAttendanceMode"
      : "https://schema.org/OfflineEventAttendanceMode",
    organizer: { "@type": "Organization", name: host.name },
    offers: {
      "@type": "Offer",
      price: event.price === "Free" ? "0" : event.price,
      priceCurrency: "USD",
      availability: "https://schema.org/InStock",
      url: eventUrl,
    },
  };

  return (
    <main className="flex flex-col min-h-screen bg-base">
      <EventPageView eventId={event.id} />
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(breadcrumbJsonLd) }}
      />
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(eventJsonLd) }}
      />
      <Navbar />

      <div className="flex-1 w-full max-w-[1221px] mx-auto px-6 py-6 sm:py-12">
        <Breadcrumb
          className="mb-6 sm:mb-8"
          items={[
            { label: "Home", href: "/" },
            { label: "Discover", href: "/discover" },
            { label: truncateTitle(event.title) },
          ]}
        />

        <div className="flex flex-col lg:flex-row gap-8 lg:gap-16">
          {/* LEFT COLUMN (Desktop) / TOP ITEMS (Mobile) */}
          <div className="lg:w-[55%] flex flex-col gap-8 lg:gap-10">
            {/* Cover Image Container - Dark Navy Background (Hero Asset: Priority Maintained) */}
            <div className="relative aspect-16/10 sm:aspect-16/11 w-full rounded-[32px] sm:rounded-[40px] overflow-hidden bg-dark shadow-sm flex items-center justify-center p-6 sm:p-12">
              <div className="relative w-full h-full">
                <Image
                  src={event.imageUrl}
                  alt={event.title}
                  fill
                  unoptimized
                  className="object-contain"
                />
              </div>
            </div>

            {/* Hosted By */}
            <div className="flex flex-col gap-4">
              <h2 className="text-xl font-bold text-black font-heading">
                <T ns="eventDetail" k="hostedBy" />
              </h2>
              <div className="flex items-center gap-3">
                <div className="relative w-8 h-8 rounded-full border border-black overflow-hidden bg-white">
                  <Image
                    src={host.avatar}
                    fill
                    alt="Stellar"
                    className="object-contain p-1.5"
                    loading="lazy"
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
                    loading="lazy"
                  />
                </div>
                <h2 className="text-xl font-bold text-black font-heading">
                  <T ns="eventDetail" k="location" />
                </h2>
              </div>
              <p className="text-[18px] font-medium text-black -mt-2">
                {event.location}
              </p>
              {event.location && (
                <a
                  href={`https://www.google.com/maps/dir/?api=1&destination=${encodeURIComponent(event.location)}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-sm text-accent hover:underline underline-offset-2"
                >
                  Get directions
                </a>
              )}
              <div className="relative aspect-16/10 w-full rounded-[24px] overflow-hidden border border-black/10">
                <MapClient location={event.location} category={event.category} title={event.title} price={event.price} imageUrl={event.imageUrl} />
              </div>
            </div>
          </div>

          {/* RIGHT COLUMN (Desktop) / BOTTOM ITEMS (Mobile) */}
          <div className="lg:w-[45%] flex flex-col gap-8 lg:gap-10">
            {/* Title */}
            <div className="flex items-start justify-between gap-4">
              <h1 className="text-[36px] sm:text-[56px] font-bold leading-[1.1] text-black font-heading">
                {event.title}
              </h1>
              <div className="mt-2">
                <ShareButton title={event.title} text={event.description} />
              </div>
            </div>

            {/* Details (Location & Date) */}
            <div className="flex flex-col gap-6">
              <div className="flex items-center gap-4">
                <div className="w-11 h-11 rounded-full border border-black flex items-center justify-center shrink-0">
                  <Image
                    src="/icons/location.svg"
                    width={22}
                    height={22}
                    alt="location"
                    loading="lazy"
                  />
                </div>
                <span className="text-[18px] sm:text-[19px] font-medium text-black">
                  {event.location}
                </span>
              </div>
              <div className="flex items-start justify-between gap-4 flex-wrap">
                <div className="flex items-start gap-4">
                  <div className="w-11 h-11 rounded-full border border-black flex items-center justify-center shrink-0">
                    <Image
                      src="/icons/notification.svg"
                      width={22}
                      height={22}
                      alt="Date"
                      loading="lazy"
                    />
                  </div>
                  {event.startsAt ? (
                    <EventTimeDisplay startsAt={event.startsAt} />
                  ) : (
                    <span className="text-[18px] sm:text-[19px] font-medium text-black">
                      {event.date}
                    </span>
                  )}
                </div>
                <AddToCalendar event={event} />
              </div>
            </div>

            {/* Registration Box */}
            <RegistrationBox event={event} host={host} />

            {/* About Section */}
            <div className="flex flex-col gap-6 pt-4">
              <h2 className="text-[20px] sm:text-[22px] font-bold text-black font-heading">
                <T ns="eventDetail" k="aboutEvent" />
              </h2>
              <div className="whitespace-pre-line text-[16px] sm:text-[17px] text-black leading-relaxed font-normal">
                {event.description || "The organizer has not added a description yet."}
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
                    loading="lazy"
                  />
                </div>
                <h2 className="text-xl font-bold text-black font-heading">
                  Location
                </h2>
              </div>
              <p className="text-[17px] font-medium text-black -mt-2">
                {event.location}
              </p>
              {event.location && (
                <a
                  href={`https://www.google.com/maps/dir/?api=1&destination=${encodeURIComponent(event.location)}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-sm text-accent hover:underline underline-offset-2"
                >
                  Get directions
                </a>
              )}
              <div className="relative aspect-16/10 w-full rounded-[24px] overflow-hidden border border-black/10">
                <MapClient location={event.location} category={event.category} title={event.title} price={event.price} imageUrl={event.imageUrl} />
              </div>
            </div>
          </div>
        </div>

        {/* Secondary Marketplace Section */}
        <div className="w-full mt-16 sm:mt-20">
          <div className="flex items-center gap-3 mb-6 sm:mb-8">
            <div className="w-1 h-8 bg-accent rounded-full" aria-hidden="true" />
            <h2 className="text-[22px] sm:text-[24px] font-bold text-black font-heading">
              Secondary Marketplace
            </h2>
          </div>
          <SecondaryMarketplaceTab eventId={eventId} />
        </div>

        {/* Report Event Link */}
        <div className="w-full mt-8 flex justify-start">
          <ReportEventLink eventId={event.id} />
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
          loading="lazy"
        />
      </div>
    </main>
  );
}