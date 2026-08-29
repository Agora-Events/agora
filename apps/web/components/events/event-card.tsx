import Image from "next/image";
import Link from "next/link";
import { LazyImage } from "@/components/ui/LazyImage";
import { useIsMounted } from "@/hooks/useIsMounted";
import { getRelativeTime } from "@/utils/relative-time";

/**
 * SOLUTION FOR ISSUE #449 & #711:
 * 1. Fluid Width: Changed `max-w-147.5` to `w-full sm:max-w-147.5`.
 * 2. Scaling: Responsive image container using `w-[40%] sm:w-auto`.
 * 3. Shadow Management: Reduced shadow depth on mobile to prevent clipping.
 * 4. Text Handling: Added `min-w-0` and `break-words` to ensure long titles don't push the container width.
 * 5. Lazy Loading (#711): Added loading="lazy" to non-hero card event thumbnails to optimize LCP.
 */

type EventCardProps = {
  id: string | number;
  title: string;
  date: string;
  location: string;
  price: string;
  imageUrl?: string;
  /** When true, shows a small loading spinner overlay in the card */
  loading?: boolean;
  isSoldOut?: boolean;
  isFollowersOnly?: boolean;
  badge?: string;
  /** ISO date string for computing relative time */
  startsAt?: string;
};

export function EventCard({
  id,
  title,
  date,
  location,
  price,
  imageUrl,
  loading = false,
  isSoldOut = false,
  isFollowersOnly = false,
  badge,
  startsAt,
}: EventCardProps) {
  const isMounted = useIsMounted();
  const locationImageSrc = location.toLowerCase().includes("discord")
    ? "/icons/discord.svg"
    : "/icons/location.svg";

  const relativeTime = isMounted && startsAt ? getRelativeTime(new Date(startsAt)) : null;

  const isFree = price.toLowerCase() === "free";
  const isSoldOutState = isSoldOut || price.toLowerCase() === "sold out" || price.toLowerCase() === "sold-out";
  const priceLabel = isSoldOutState
    ? "Sold Out"
    : isFree
    ? "Free"
    : price.startsWith("$")
    ? price
    : `$${price}`;

  const activeBadge = badge || (isSoldOutState ? "Sold Out" : isFollowersOnly ? "Followers Only" : null);
  const displayImage = imageUrl || "/images/event-placeholder.png";

  return (
    <Link href={`/events/${id}`} className="block w-full">
      <div className="w-full sm:max-w-147.5 shadow-[-6px_6px_0_rgba(0,0,0,1)] sm:shadow-[-9px_9px_0_rgba(0,0,0,1)] flex flex-col bg-surface pb-4.75 sm:pl-12.5 pl-4 pt-5 sm:pt-9.75 rounded-xl sm:pr-5 pr-3.75 transition-transform hover:scale-[1.02] overflow-hidden relative">
        {/* Loading overlay spinner */}
        {loading && (
          <div className="absolute inset-0 bg-white/60 z-10 flex items-center justify-center rounded-xl">
            <div className="w-8 h-8 border-3 border-accent border-t-transparent rounded-full animate-spin" />
          </div>
        )}
        <div className="flex gap-4.75">
          {/* Left Side: Image & Mobile Actions */}
          <div className="flex-shrink-0 w-[40%] sm:w-auto relative">
            {activeBadge && (
              <span className={`absolute top-2 left-2 z-10 px-2 py-0.5 text-[10px] font-bold rounded-md border border-black shadow-[1px_1px_0px_0px_rgba(0,0,0,1)] ${
                isSoldOutState ? "bg-red-500 text-white" : isFollowersOnly ? "bg-purple-500 text-white" : "bg-yellow-400 text-black"
              }`}>
                {activeBadge}
              </span>
            )}
            {displayImage ? (
              <LazyImage
                src={displayImage}
                alt={title}
                width={227}
                height={112}
                className="object-cover w-full h-auto rounded-lg min-h-[112px] bg-gray-100"
              />
            ) : (
              <div className="w-[227px] h-[112px] rounded-lg bg-gray-200 border border-black/10 flex items-center justify-center text-gray-400 text-xs font-medium">
                No Image
              </div>
            )}
            
            {/* Price Label (Mobile Only) */}
            <div className="flex justify-center font-semibold sm:hidden text-[10px]/2.5 mt-4">
              {priceLabel}
            </div>
            
            {/* View Event (Mobile Only) */}
            <div className="sm:hidden justify-center flex items-center gap-1 mt-1.5 text-black text-[12px]/7.5 font-medium cursor-pointer">
              <span className="whitespace-nowrap">View Event</span>
              <Image
                src="/icons/arrow-right.svg"
                width={18}
                height={18}
                alt="arrow right"
                className="object-contain"
                loading="lazy"
              />
            </div>
          </div>

          {/* Right Side: Content */}
          <div className="flex flex-col grow justify-between sm:justify-start min-w-0">
            {/* Date (Desktop) */}
            <div className="hidden sm:block">
              <span className="font-light text-[15px]/7.5">
                {date}
              </span>
              {relativeTime && (
                <span className="font-normal text-[13px]/7.5 text-gray-600 block">
                  {relativeTime}
                </span>
              )}
            </div>

            {/* Title: Prevent overflow with break-words */}
            <p className="font-semibold text-[14px] sm:text-[15px]/5 mt-1 sm:mt-2.5 break-words leading-tight">
              {title}
            </p>

            {/* Price Label (Desktop) */}
            <div className="max-sm:hidden pr-3 font-semibold sm:text-[13px]/3.25 text-[10px]/2.5 mt-2 self-end">
              {priceLabel}
            </div>

            <div>
              {/* Date (Mobile) */}
              <div className="max-sm:block hidden">
                <span className="font-light text-[11px] sm:text-[12px]/7.5">
                  {date}
                </span>
                {relativeTime && (
                  <span className="font-normal text-[10px]/7.5 text-gray-600 block">
                    {relativeTime}
                  </span>
                )}
              </div>

              {/* Location */}
              <div className="flex items-center gap-1.25 mt-1">
                <Image
                  src={locationImageSrc}
                  alt={location.toLowerCase().includes("discord") ? "Discord" : "Location"}
                  width={16}
                  height={16}
                  className="object-contain flex-shrink-0"
                  loading="lazy"
                />
                <span className="font-normal text-[11px] sm:text-[12px]/7.5 line-clamp-1">
                  {location}
                </span>
              </div>
            </div>
          </div>
        </div>

        {/* View Event (Desktop Only) */}
        <div className="self-end hidden sm:flex mr-6 gap-1.5 mt-1.5 text-black text-[12px]/7.5 font-medium cursor-pointer">
          View Event
          <Image
            src="/icons/arrow-right.svg"
            width={24}
            height={24}
            alt="arrow-right icon"
            className="object-cover"
            loading="lazy"
          />
        </div>
      </div>
    </Link>
  );
}