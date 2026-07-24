"use client";

import group from "../../public/icons/user-group.svg";
import left from "../../public/icons/arrow-left.svg";
import right from "../../public/icons/arrow-right.svg";
import Image from "next/image";
import { useEffect, useRef, useState } from "react";
import { fetchOrganizers, type DiscoverOrganizer } from "@/utils/api";
import { Button } from "@/components/ui/button";
import Link from "next/link";

const fallbackCardsData: DiscoverOrganizer[] = [
  {
    id: "stellar-west-africa",
    title: "Stellar West Africa",
    description:
      "Building and empowering the Stellar ecosystem in West Africa through education, developer support, and real-world blockchain adoption.",
    image: "/icons/stellar-west-africa.svg",
  },
  {
    id: "stellar-east-african-community",
    title: "Stellar East African Community",
    description:
      "Building and empowering the Stellar ecosystem in East Africa through education, developer support, and real-world blockchain adoption.",
    image: "/icons/stellar-east-africa.svg",
  },
  {
    id: "stellar-india",
    title: "Stellar India",
    description:
      "Building and empowering the Stellar ecosystem in West Africa through education, developer support, and real-world blockchain adoption.",
    image: "/icons/stellar-india.svg",
  },
  {
    id: "stellar-portugal",
    title: "Stellar Portugal",
    description:
      "Building and empowering the Stellar ecosystem in West Africa through education, developer support, and real-world blockchain adoption.",
    image: "/icons/stellar-portugal.svg",
  },
];

function SubscribeButton() {
  return (
    <Button
      variant="primary"
      className="absolute top-40 right-5 rounded-lg px-3 py-2"
    >
      <Image src={group} alt="User Group Icon" className="w-8 h-8" />
      Subscribe
    </Button>
  );
}

type OrganizerComponentProps = {
  onError: (message: string) => void;
};

export function OrganizerComponent({ onError }: OrganizerComponentProps) {
  const cardsRef = useRef<HTMLDivElement>(null);
  const [cardsData, setCardsData] = useState<DiscoverOrganizer[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // AbortController cancels the in-flight fetch when the component unmounts,
    // preventing state updates on an unmounted component and avoiding memory leaks.
    const controller = new AbortController();

    const loadOrganizers = async () => {
      try {
        const data = await fetchOrganizers(controller.signal);
        setCardsData(data);
      } catch (err) {
        // Ignore abort errors — they are intentional and not user-facing.
        if (err instanceof Error && err.name === "AbortError") return;
        setCardsData([]);
        onError("Could not load organizers");
      } finally {
        // Only update loading state if the fetch was not aborted.
        if (!controller.signal.aborted) {
          setIsLoading(false);
        }
      }
    };

    loadOrganizers();
    return () => {
      controller.abort();
    };
  }, [onError]);

  const organizersToRender = cardsData.length > 0 ? cardsData : fallbackCardsData;

  const scrollLeft = () => {
    cardsRef.current?.scrollBy({ left: -300, behavior: "smooth" });
  };

  const scrollRight = () => {
    cardsRef.current?.scrollBy({ left: 300, behavior: "smooth" });
  };

  return (
    <div className="p-10 pl-45 hidden lg:block bg-base">
      <div className="flex justify-start items-center gap-4 p-5 pb-10">
        <h1 className="font-semibold md:text-4xl pl-3">Explore organizers</h1>
        <Image
          src={group}
          alt="User Group Icon"
          className="w-7 h-7 font-bold mt-2"
        />
      </div>
      <section
        className="flex justify-center items-center gap-10 overflow-x-auto h-65 pl-75 mr-50"
        ref={cardsRef}
      >
        {isLoading &&
          Array.from({ length: 3 }).map((_, index) => (
            <div
              key={`organizer-skeleton-${index}`}
              className="h-58 min-w-100 animate-pulse rounded-2xl border border-black/20 bg-black/10"
            />
          ))}
        {!isLoading &&
          organizersToRender.map((card) => (
          <Link key={card.id} href={`/organizers/${card.id}`} className="relative h-full block">
            <section className="absolute border-10 rounded-2xl bg-yellow-400 border-yellow-400 w-102 h-58 -left-2 top-2 z-0"></section>
            <div
              className="relative z-10 bg-black text-white p-5x border rounded-2xl lg:min-w-100
                     h-40 lg:h-58 cursor-pointer hover:-translate-y-1 transition-transform"
            >
              <div className="absolute top-5 left-5">
                <Image
                  src={card.image}
                  alt={card.title}
                  height={65}
                  width={65}
                  className="relative z-10 border-4 border-black rounded-full object-cover"
                />
                <div className="absolute -left-1 top-1 w-15 h-15 bg-white rounded-full z-0" />
              </div>
              <div className="text-lg font-semibold absolute left-25 top-10 w-full hover:underline">
                {card.title}
              </div>
              <p className="text-xs absolute left-25 top-20 w-65">
                {card.description}
              </p>
              <div onClick={(e) => e.preventDefault()}>
                <SubscribeButton />
              </div>
            </div>
          </Link>
          ))}
        {!isLoading && cardsData.length === 0 && (
          <p className="text-sm text-black/60">No data available</p>
        )}
      </section>
      <span className="flex justify-end gap-5 pr-50 pt-5">
        <Image
          src={left}
          alt="Left Arrow"
          className="w-12 h-12 p-3 hover:cursor-pointer bg-surface rounded-full"
          onClick={scrollLeft}
        />
        <Image
          src={right}
          alt="Right Arrow"
          className="w-12 h-12 p-3 hover:cursor-pointer bg-surface rounded-full"
          onClick={scrollRight}
        />
      </span>
    </div>
  );
}
