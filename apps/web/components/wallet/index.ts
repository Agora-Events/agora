/**
 * Wallet feature components.
 *
 * Re-export every component from this barrel so consumers can import from a
 * single path:
 *
 * ```ts
 * import { TicketCard, PastEventsSection, PoapCard } from "@/components/wallet";
 * ```
 */

export { TicketCard } from "./TicketCard";
export type { TicketCardProps, TicketStatus } from "./TicketCard";

export { PoapCard } from "./PoapCard";
export type { PoapCollectible, PoapCardProps } from "./PoapCard";

export { PastEventsSection } from "./PastEventsSection";
export type { PastEventsSectionProps } from "./PastEventsSection";
