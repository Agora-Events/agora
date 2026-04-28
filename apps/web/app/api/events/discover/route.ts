import { NextResponse } from "next/server";
import { listEvents } from "@/lib/events-store";

export async function GET() {
  const events = listEvents();

  const categories = Array.from(
    new Set(events.map((event) => event.category)),
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
      date: new Date(event.startsAt).toLocaleString(),
      location: event.location,
      price: event.ticketPrice === 0 ? "Free" : String(event.ticketPrice),
      imageUrl: event.imageUrl,
      category: event.category,
    }));

  const organizers = Array.from(
    events.reduce((acc, event) => {
      if (!acc.has(event.organizerName)) {
        acc.set(event.organizerName, {
          id: event.organizerName.toLowerCase().replace(/\s+/g, "-"),
          title: event.organizerName,
          description: `Organizer of ${event.category} events on Agora.`,
          image: "/icons/stellar-west-africa.svg",
        });
      }
      return acc;
    }, new Map<string, { id: string; title: string; description: string; image: string }>()),
  ).map(([, value]) => value);

  return NextResponse.json({ categories, popularEvents, organizers });
}
