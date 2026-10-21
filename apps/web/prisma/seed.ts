/**
 * Local development seed script.
 *
 * The in-memory mock data in `lib/events-store.ts` never reaches Postgres,
 * so a fresh checkout has an empty database and pages like the wallet view
 * or the organizer analytics dashboard have nothing to render.
 *
 * This script seeds a small, representative dataset directly through
 * Prisma: ~10 events across several categories spanning past and future
 * dates, a few organizer profiles, and a handful of tickets.
 *
 * All records use fixed ids and are written with `upsert`, so running this
 * script multiple times (e.g. after `prisma migrate reset`) is safe and
 * will not create duplicates.
 *
 * Usage:
 *   pnpm --filter web exec prisma db seed
 *   (or, from apps/web:  npx prisma db seed)
 */
import { PrismaClient } from "@prisma/client";

const prisma = new PrismaClient();

/** Builds a deterministic, Stellar-looking public key (56 chars, starts with "G"). */
function wallet(seed: string): string {
  return `G${seed.toUpperCase()}`.padEnd(56, "X").slice(0, 56);
}

const ORGANIZER_WALLETS = {
  stellarWestAfrica: wallet("STELLARWESTAFRICA"),
  agoraBuilders: wallet("AGORABUILDERS"),
  fintechOrbit: wallet("FINTECHORBIT"),
};

const organizerProfiles = [
  {
    address: ORGANIZER_WALLETS.stellarWestAfrica,
    displayName: "Stellar West Africa",
    bio: "Community-run meetups and workshops for builders on Stellar.",
    avatarUrl: "/images/organizers/stellar-west-africa.png",
    socials: { twitter: "https://x.com/stellarwestafrica" },
  },
  {
    address: ORGANIZER_WALLETS.agoraBuilders,
    displayName: "Agora Builders",
    bio: "Networking and demo nights for the local ecosystem.",
    avatarUrl: "/images/organizers/agora-builders.png",
    socials: { website: "https://agora.dev" },
  },
  {
    address: ORGANIZER_WALLETS.fintechOrbit,
    displayName: "Fintech Orbit",
    bio: "Panels and summits on the future of payments.",
    avatarUrl: "/images/organizers/fintech-orbit.png",
    socials: { twitter: "https://x.com/fintechorbit" },
  },
];

// Fixed ids keep this script idempotent across repeated runs.
const events = [
  {
    id: "seed-event-dev-meetup-past",
    title: "Stellar Developer Meetup",
    description: "Hands-on workshop for building with Stellar tooling.",
    startsAt: new Date("2026-05-01T18:00:00.000Z"), // past
    location: "Lagos",
    category: "Tech",
    organizerName: "Stellar West Africa",
    organizerWallet: ORGANIZER_WALLETS.stellarWestAfrica,
    imageUrl: "/images/event1.png",
    ticketPrice: 0,
    totalTickets: 200,
    mintedTickets: 178,
    hostEmail: "host@agora.dev",
  },
  {
    id: "seed-event-builder-night-past",
    title: "Community Builder Night",
    description: "Networking event for local ecosystem builders.",
    startsAt: new Date("2026-03-01T17:00:00.000Z"), // past
    location: "Online",
    category: "Party",
    organizerName: "Agora Builders",
    organizerWallet: ORGANIZER_WALLETS.agoraBuilders,
    imageUrl: "/images/event2.png",
    ticketPrice: 15,
    totalTickets: 120,
    mintedTickets: 120,
    followersOnly: true,
    hostEmail: "community@agora.dev",
  },
  {
    id: "seed-event-payments-summit-future",
    title: "Future of Payments Summit",
    description: "Panel and demos on modern payment rails.",
    startsAt: new Date("2026-10-20T09:00:00.000Z"), // future
    location: "London",
    category: "Crypto",
    organizerName: "Fintech Orbit",
    organizerWallet: ORGANIZER_WALLETS.fintechOrbit,
    imageUrl: "/images/event3.png",
    ticketPrice: 30,
    totalTickets: 400,
    mintedTickets: 40,
    hostEmail: "payments@agora.dev",
  },
  {
    id: "seed-event-summer-music-fest-future",
    title: "Summer Music Festival",
    description: "An open-air festival featuring local and touring acts.",
    startsAt: new Date("2026-09-12T16:00:00.000Z"), // future
    location: "Accra",
    category: "Music",
    organizerName: "Agora Builders",
    organizerWallet: ORGANIZER_WALLETS.agoraBuilders,
    imageUrl: "/images/event2.png",
    ticketPrice: 40,
    totalTickets: 500,
    mintedTickets: 210,
    hostEmail: "community@agora.dev",
  },
  {
    id: "seed-event-art-walk-past",
    title: "Downtown Art Walk",
    description: "Self-guided tour through local galleries and pop-up shows.",
    startsAt: new Date("2026-02-14T15:00:00.000Z"), // past
    location: "Nairobi",
    category: "Art",
    organizerName: "Stellar West Africa",
    organizerWallet: ORGANIZER_WALLETS.stellarWestAfrica,
    imageUrl: "/images/event1.png",
    ticketPrice: 0,
    totalTickets: 150,
    mintedTickets: 96,
    hostEmail: "host@agora.dev",
  },
  {
    id: "seed-event-charity-run-future",
    title: "Charity 5K Fun Run",
    description: "A community run supporting local education initiatives.",
    startsAt: new Date("2026-11-08T07:00:00.000Z"), // future
    location: "Kigali",
    category: "Sports",
    organizerName: "Agora Builders",
    organizerWallet: ORGANIZER_WALLETS.agoraBuilders,
    imageUrl: "/images/event2.png",
    ticketPrice: 10,
    totalTickets: 300,
    mintedTickets: 55,
    hostEmail: "community@agora.dev",
  },
  {
    id: "seed-event-food-festival-future",
    title: "Street Food Festival",
    description: "A weekend celebration of regional street food vendors.",
    startsAt: new Date("2026-09-26T11:00:00.000Z"), // future
    location: "Lagos",
    category: "Food",
    organizerName: "Stellar West Africa",
    organizerWallet: ORGANIZER_WALLETS.stellarWestAfrica,
    imageUrl: "/images/event1.png",
    ticketPrice: 5,
    totalTickets: 600,
    mintedTickets: 312,
    hostEmail: "host@agora.dev",
  },
  {
    id: "seed-event-founders-forum-past",
    title: "Founders Forum",
    description: "A closed-door working session for early-stage founders.",
    startsAt: new Date("2026-04-18T13:00:00.000Z"), // past
    location: "Cape Town",
    category: "Business",
    organizerName: "Fintech Orbit",
    organizerWallet: ORGANIZER_WALLETS.fintechOrbit,
    imageUrl: "/images/event3.png",
    ticketPrice: 50,
    totalTickets: 80,
    mintedTickets: 80,
    followersOnly: true,
    hostEmail: "payments@agora.dev",
  },
  {
    id: "seed-event-wellness-retreat-future",
    title: "Mindfulness & Wellness Retreat",
    description: "A day of guided sessions on stress management and health.",
    startsAt: new Date("2026-12-05T08:00:00.000Z"), // future
    location: "Zanzibar",
    category: "Health",
    organizerName: "Agora Builders",
    organizerWallet: ORGANIZER_WALLETS.agoraBuilders,
    imageUrl: "/images/event2.png",
    ticketPrice: 60,
    totalTickets: 100,
    mintedTickets: 18,
    hostEmail: "community@agora.dev",
  },
  {
    id: "seed-event-web3-bootcamp-future",
    title: "Web3 Bootcamp",
    description: "A three-day intensive on building on Soroban and Stellar.",
    startsAt: new Date("2026-10-05T09:00:00.000Z"), // future
    location: "Online",
    category: "Education",
    organizerName: "Stellar West Africa",
    organizerWallet: ORGANIZER_WALLETS.stellarWestAfrica,
    imageUrl: "/images/event1.png",
    ticketPrice: 0,
    totalTickets: 1000,
    mintedTickets: 430,
    hostEmail: "host@agora.dev",
  },
];

const tickets = [
  {
    id: "seed-ticket-01",
    eventId: "seed-event-dev-meetup-past",
    buyerWallet: wallet("BUYERALICE"),
    ownerWallet: wallet("BUYERALICE"),
    quantity: 2,
    utmSource: "twitter",
    utmMedium: "social",
    utmCampaign: "dev-meetup-launch",
  },
  {
    id: "seed-ticket-02",
    eventId: "seed-event-dev-meetup-past",
    buyerWallet: wallet("BUYERBOB"),
    ownerWallet: wallet("BUYERBOB"),
    quantity: 1,
  },
  {
    id: "seed-ticket-03",
    eventId: "seed-event-builder-night-past",
    buyerWallet: wallet("BUYERCARLA"),
    ownerWallet: wallet("BUYERCARLA"),
    quantity: 3,
    utmSource: "newsletter",
    utmMedium: "email",
    utmCampaign: "builder-night",
  },
  {
    id: "seed-ticket-04",
    eventId: "seed-event-payments-summit-future",
    buyerWallet: wallet("BUYERDAVID"),
    ownerWallet: wallet("BUYERDAVID"),
    quantity: 1,
  },
  {
    id: "seed-ticket-05",
    eventId: "seed-event-summer-music-fest-future",
    buyerWallet: wallet("BUYERELLA"),
    // Gifted to a different wallet than the buyer.
    ownerWallet: wallet("RECIPIENTFRANK"),
    quantity: 2,
    utmSource: "instagram",
    utmMedium: "social",
    utmCampaign: "summer-music-fest",
  },
  {
    id: "seed-ticket-06",
    eventId: "seed-event-web3-bootcamp-future",
    buyerWallet: wallet("BUYERGRACE"),
    ownerWallet: wallet("BUYERGRACE"),
    quantity: 1,
  },
];

const analyticsEvents = [
  { id: "seed-analytics-01", eventId: "seed-event-dev-meetup-past", type: "view" },
  { id: "seed-analytics-02", eventId: "seed-event-dev-meetup-past", type: "purchase" },
  { id: "seed-analytics-03", eventId: "seed-event-summer-music-fest-future", type: "view" },
  { id: "seed-analytics-04", eventId: "seed-event-summer-music-fest-future", type: "purchase" },
  { id: "seed-analytics-05", eventId: "seed-event-web3-bootcamp-future", type: "view" },
];

async function main() {
  for (const profile of organizerProfiles) {
    await prisma.organizerProfile.upsert({
      where: { address: profile.address },
      update: profile,
      create: profile,
    });
  }
  console.log(`Seeded ${organizerProfiles.length} organizer profiles.`);

  for (const event of events) {
    await prisma.event.upsert({
      where: { id: event.id },
      update: event,
      create: event,
    });
  }
  console.log(`Seeded ${events.length} events.`);

  for (const ticket of tickets) {
    await prisma.ticket.upsert({
      where: { id: ticket.id },
      update: ticket,
      create: ticket,
    });
  }
  console.log(`Seeded ${tickets.length} tickets.`);

  for (const analyticsEvent of analyticsEvents) {
    await prisma.analyticsEvent.upsert({
      where: { id: analyticsEvent.id },
      update: analyticsEvent,
      create: analyticsEvent,
    });
  }
  console.log(`Seeded ${analyticsEvents.length} analytics events.`);
}

main()
  .catch((error) => {
    console.error("Seed failed:", error);
    process.exitCode = 1;
  })
  .finally(async () => {
    await prisma.$disconnect();
  });
