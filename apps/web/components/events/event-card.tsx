import Image from "next/image";
import Link from "next/link";

/**
 * SOLUTION FOR ISSUE #449:
 * 1. Fluid Width: Changed `max-w-147.5` to `w-full sm:max-w-147.5`.
 * 2. Scaling: Responsive image container using `w-[40%] sm:w-auto`.
 * 3. Shadow Management: Reduced shadow depth on mobile to prevent clipping.
 * 4. Text Handling: Added `min-w-0` and `break-words` to ensure long titles don't push the container width.
 */

type EventCardProps = {
  id: string | number;
  title: string;
  date: string;
  location: string;
  price: string;
  imageUrl: string;
};

export function EventCard({
  id,
  title,
  date,
  location,
  price,
  imageUrl,
}: EventCardProps) {
  const locationImageSrc = location.toLowerCase().includes("discord")
    ? "/icons/discord.svg"
    : "/icons/location.svg";

  const priceLabel = price.toLowerCase() === "free" ? "Free" : `$${price}`;

  return (
    <Link href={`/events/${id}`} className="block w-full">
      {/* Container Fix: 
          - w-full ensures it doesn't overflow 375px.
          - sm:max-w-147.5 preserves original desktop size.
          - Shadow reduced from -9 to -6 on mobile to avoid viewport bleeding.
      */}
      <div className="w-full sm:max-w-147.5 shadow-[-6px_6px_0_rgba(0,0,0,1)] sm:shadow-[-9px_9px_0_rgba(0,0,0,1)] flex flex-col bg-[#FFEFD3] pb-4.75 sm:pl-12.5 pl-4 pt-5 sm:pt-9.75 rounded-xl sm:pr-5 pr-3.75 transition-transform hover:scale-[1.02] overflow-hidden">
        <div className="flex gap-4.75">
          {/* Left Side: Image & Mobile Actions */}
          <div className="flex-shrink-0 w-[40%] sm:w-auto">
            <Image
              src={imageUrl}
              width={227}
              height={112}
              alt="event image"
              className="object-cover w-full h-auto rounded-lg"
            />
            
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
              />
            </div>
          </div>

          {/* Right Side: Content */}
          <div className="flex flex-col grow justify-between sm:justify-start min-w-0">
            {/* Date (Desktop) */}
            <span className="font-light text-[15px]/7.5 hidden sm:block">
              {date}
            </span>

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
              <span className="font-light max-sm:block hidden text-[11px] sm:text-[12px]/7.5">
                {date}
              </span>

              {/* Location */}
              <div className="flex items-center gap-1.25 mt-1">
                <Image
                  src={locationImageSrc}
                  alt="location"
                  width={16}
                  height={16}
                  className="object-contain flex-shrink-0"
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
          />
        </div>
      </div>
    </Link>
  );
}