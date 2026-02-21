import { TicketTierCard } from "../ui/refundability-badge";

export type TicketTier = {
  id: string;
  name: string;
  price: number;
  description?: string;
  isRefundable: boolean;
  totalQuantity: number;
  availableQuantity: number;
};

type TicketTiersSectionProps = {
  tiers: TicketTier[];
  eventId: string;
  onTierSelect?: (tier: TicketTier) => void;
};

/**
 * Displays all ticket tiers for an event with refundability indicators.
 * Helps guests understand refund policies before purchase.
 */
export function TicketTiersSection({
  tiers,
  eventId,
  onTierSelect,
}: TicketTiersSectionProps) {
  if (!tiers || tiers.length === 0) {
    return (
      <div className="text-center py-8 text-gray-600">
        No ticket tiers available for this event.
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold text-gray-900 mb-6">
        Choose Your Ticket Tier
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {tiers.map((tier) => (
          <TicketTierCard
            key={tier.id}
            id={tier.id}
            name={tier.name}
            price={tier.price}
            description={tier.description}
            isRefundable={tier.isRefundable}
            quantity={tier.totalQuantity}
            availableQuantity={tier.availableQuantity}
            onSelect={() => onTierSelect?.(tier)}
          />
        ))}
      </div>

      {tiers.some((t) => !t.isRefundable) && (
        <div className="mt-8 p-4 bg-blue-50 border border-blue-200 rounded-lg">
          <p className="text-sm text-blue-900">
            <strong>Note:</strong> Some ticket tiers are non-refundable. Please
            review the refund policy for each tier before purchase. Organizer
            cancellations will result in refunds for all tickets, regardless of
            tier refundability.
          </p>
        </div>
      )}
    </div>
  );
}
