import { NextRequest, NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import { type Event, Prisma } from "@prisma/client";
import { withErrorHandler } from "@/lib/api-handler";

export const dynamic = "force-dynamic";

type OrganizerData = {
  id: string;
  title: string;
  description: string;
  image: string;
  wallet?: string;
};

export const GET = withErrorHandler(async (request: NextRequest) => {
  const { searchParams } = new URL(request.url);
  const category = searchParams.get("category");
  const free = searchParams.get("free") === "1" || searchParams.get("free") === "true";

  const where: Prisma.EventWhereInput = {};
  if (category) {
    where.category = category;
  }
  if (free) {
    where.ticketPrice = 0;
  }

  const events = await prisma.event.findMany({ where });

  const categories = Array.from(
    new Set<string>(events.map((event: Event) => event.category))
  ).map((name) => ({
    name,
    icon: `/icons/${name.toLowerCase()}.svg`,
    color: "#DBF4B9",
  }));

  const popularEvents = [...events]
    .sort((a, b) => b.mintedTickets - a.mintedTickets)
    .slice(0, 8)
    .map((event) => ({
      id: event.id,
      title: event.title,
      date: event.startsAt.toLocaleString(),
      startsAt: event.startsAt.toISOString(),
      location: event.location,
      price: event.ticketPrice === 0 ? "Free" : String(event.ticketPrice),
      imageUrl: event.imageUrl,
      category: event.category,
      mintedTickets: event.mintedTickets,
    }));

  const organizers = Array.from(
    events.reduce((acc: Map<string, OrganizerData>, event: Event) => {
      if (!acc.has(event.organizerName)) {
        acc.set(event.organizerName, {
          id: event.organizerName.toLowerCase().replace(/\s+/g, "-"),
          title: event.organizerName,
          description: `Organizer of ${event.category} events on Agora.`,
          image: "/icons/stellar-west-africa.svg",
          wallet: event.organizerWallet,
        });
      }
      return acc;
    }, new Map<string, OrganizerData>()),
  ) as [string, OrganizerData][];

  const organizerList = organizers.map(([, organizer]) => organizer);

  return NextResponse.json({ categories, popularEvents, organizers: organizerList });
});


