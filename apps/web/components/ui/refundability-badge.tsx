import Image from "next/image";

type RefundabilityIndicatorProps = {
  isRefundable: boolean;
  size?: "small" | "medium" | "large";
};

/**
 * Displays a visual indicator for ticket tier refundability status.
 * Shows "Non-refundable" badge only when isRefundable is false.
 * Default state (refundable) has no indicator as per UX spec.
 */
export function RefundabilityIndicator({
  isRefundable,
  size = "medium",
}: RefundabilityIndicatorProps) {
  if (isRefundable) {
    return null;
  }

  const sizeClasses = {
    small: "text-xs px-2 py-1",
    medium: "text-sm px-3 py-1.5",
    large: "text-base px-4 py-2",
  };

  const iconSize = size === "small" ? 16 : size === "medium" ? 20 : 24;

  return (
    <div
      className={`
        flex items-center gap-2 rounded-md
        ${sizeClasses[size]}
        bg-amber-50 border border-amber-300 text-amber-900
      `}
      role="status"
      aria-label="Non-refundable ticket"
    >
      <Image
        src="/icons/warning.svg"
        alt="warning"
        width={iconSize}
        height={iconSize}
        className="shrink-0"
      />
      <span className="font-medium">Non-refundable</span>
    </div>
  );
}

type TicketTierCardProps = {
  id: string;
  name: string;
  price: string | number;
  description?: string;
  isRefundable: boolean;
  quantity?: number;
  availableQuantity?: number;
  onSelect?: () => void;
};

/**
 * Displays a ticket tier with refundability status.
 * Includes pricing, description, and visual refund policy indicator.
 */
export function TicketTierCard({
  id,
  name,
  price,
  description,
  isRefundable,
  quantity,
  availableQuantity,
  onSelect,
}: TicketTierCardProps) {
  const priceDisplay =
    typeof price === "string" ? price : `$${price.toFixed(2)}`;
  const isSoldOut = availableQuantity === 0;

  return (
    <div
      className={`
        p-4 border rounded-lg transition-all
        ${
          isSoldOut
            ? "opacity-60 cursor-not-allowed bg-gray-50"
            : "hover:shadow-md cursor-pointer bg-white border-gray-200"
        }
      `}
      onClick={!isSoldOut ? onSelect : undefined}
      role="button"
      tabIndex={isSoldOut ? -1 : 0}
      aria-disabled={isSoldOut}
    >
      <div className="flex justify-between items-start mb-3">
        <div>
          <h3 className="font-semibold text-gray-900">{name}</h3>
          {description && (
            <p className="text-sm text-gray-600 mt-1">{description}</p>
          )}
        </div>
        <div className="text-right">
          <p className="text-lg font-bold text-gray-900">{priceDisplay}</p>
          {quantity !== undefined && (
            <p className="text-xs text-gray-500 mt-1">
              {availableQuantity !== undefined
                ? `${availableQuantity}/${quantity} available`
                : `${quantity} available`}
            </p>
          )}
        </div>
      </div>

      {!isRefundable && (
        <div className="mt-3 pt-3 border-t border-gray-100">
          <RefundabilityIndicator isRefundable={isRefundable} size="small" />
        </div>
      )}

      {isSoldOut && (
        <div className="mt-3 pt-3 border-t border-gray-100">
          <p className="text-sm font-medium text-gray-600">Sold out</p>
        </div>
      )}
    </div>
  );
}
