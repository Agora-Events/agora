import Image from "next/image";
import Link from "next/link";

type EventCardProps = {
  id: number;
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
    <Link href={`/events/${id}`} className="block">
      <div className="w-full max-w-147.5 shadow-[-9px_9px_0_rgba(0,0,0,1)]  flex flex-col bg-[#FFEFD3] pb-4.75 sm:pl-12.5 pl-5.5 pt-5 sm:pt-9.75 rounded-xl  sm:pr-5 pr-3.75 transition-transform hover:scale-[1.02]">
        <div className="flex gap-4.75 ">
          <div>
            <Image
              src={imageUrl}
              width={227}
              height={112}
              alt="event image"
              className="object-cover"
            />
            <div className="flex justify-center font-semibold sm:hidden text-[10px]/2.5 mt-4 self-end">
              {priceLabel}
            </div>
            <div className="sm:hidden justify-center flex gap-1 mt-1.5 text-black text-[12px]/7.5 font-medium cursor-pointer">
              View Event
              <Image
                src="/icons/arrow-right.svg"
                width={24}
                height={24}
                alt="arrow right"
                className="object-contain"
              />
            </div>
          </div>

          <div className="flex flex-col grow max-sm:justify-between">
            <span className="font-light text-[15px]/7.5 hidden sm:block">
              {date}
            </span>
            <p className="font-semibold text-[15px]/5 mt-2.5">{title}</p>
            <div className="max-sm:hidden pr-3 font-semibold sm:text-[13px]/3.25 text-[10px]/2.5 mt-2 self-end">
              {priceLabel}
            </div>

            <div>
              <span className="font-light max-sm:block hidden text-[12px]/7.5">
                {date}
              </span>
              <div className="flex gap-1.25">
                <Image
                  src={locationImageSrc}
                  alt="location"
                  width={24}
                  height={24}
                  className="object-contain"
                />
                <span className="font-normal text-[12px]/7.5 line-clamp-1">
                  {location}
                </span>
              </div>
            </div>
          </div>
        </div>

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
