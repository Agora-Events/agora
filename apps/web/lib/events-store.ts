export type EventRecord = {
  id: string;
  title: string;
  description: string;
  startsAt: string;
  location: string;
  category: string;
  organizerName: string;
  organizerWallet: string;
  imageUrl: string;
  ticketPrice: number;
  totalTickets: number;
  mintedTickets: number;
  followersOnly?: boolean;
  hostEmail: string;
};

type CreateEventInput = {
  title: string;
  description?: string;
  startsAt: string;
  location: string;
  category: string;
  organizerName: string;
  organizerWallet: string;
  imageUrl?: string;
  ticketPrice?: number;
  totalTickets?: number;
  followersOnly?: boolean;
};

const eventsStore: EventRecord[] = [
  {
    id: "evt_1",
    title: "Stellar Developer Meetup",
    description: "Hands-on workshop for building with Stellar tooling.",
    startsAt: "2026-06-01T18:00:00.000Z",
    location: "Lagos",
    category: "Tech",
    organizerName: "Stellar West Africa",
    organizerWallet: "GDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    imageUrl: "/images/event1.png",
    ticketPrice: 0,
    totalTickets: 200,
    mintedTickets: 90,
    hostEmail: "host@agora.dev",
  },
  {
    id: "evt_2",
    title: "Community Builder Night",
    description: "Networking event for local ecosystem builders.",
    startsAt: "2026-03-01T17:00:00.000Z",
    location: "Online",
    category: "Party",
    organizerName: "Agora Builders",
    organizerWallet: "GDBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
    imageUrl: "/images/event2.png",
    ticketPrice: 15,
    totalTickets: 120,
    mintedTickets: 120,
    followersOnly: true,
    hostEmail: "community@agora.dev",
  },
  {
    id: "evt_3",
    title: "Future of Payments Summit",
    description: "Panel and demos on modern payment rails.",
    startsAt: "2026-10-20T09:00:00.000Z",
    location: "London",
    category: "Crypto",
    organizerName: "Fintech Orbit",
    organizerWallet: "GDCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
    imageUrl: "/images/event3.png",
    ticketPrice: 30,
    totalTickets: 400,
    mintedTickets: 40,
    hostEmail: "payments@agora.dev",
  },
];

let nextEventId = 4;

export function listEvents(): EventRecord[] {
  return [...eventsStore];
}

export function getEventById(id: string): EventRecord | null {
  return eventsStore.find((event) => event.id === id) || null;
}

export function createEvent(input: CreateEventInput, hostEmail: string): EventRecord {
  const event: EventRecord = {
    id: `evt_${nextEventId++}`,
    title: input.title,
    description: input.description || "",
    startsAt: input.startsAt,
    location: input.location,
    category: input.category,
    organizerName: input.organizerName,
    organizerWallet: input.organizerWallet,
    imageUrl: input.imageUrl || "/images/event1.png",
    ticketPrice: input.ticketPrice ?? 0,
    totalTickets: input.totalTickets ?? 100,
    mintedTickets: 0,
    followersOnly: Boolean(input.followersOnly),
    hostEmail,
  };

  eventsStore.unshift(event);
  return event;
}

export function incrementMintedTickets(id: string, quantity: number): EventRecord | null {
  const event = eventsStore.find((entry) => entry.id === id);
  if (!event) {
    return null;
  }

  event.mintedTickets += quantity;
  return event;
}

export function hasAvailableTickets(event: EventRecord, quantity: number): boolean {
  return event.mintedTickets + quantity <= event.totalTickets;
}
