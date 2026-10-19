"use client";

import React from "react";
import {
  useTicketAvailability,
  getAvailabilityStatus,
} from "@/hooks/useTicketAvailability";

interface TicketAvailabilityDisplayProps {
  /**
   * Event ID to monitor ticket availability for
   */
  eventId: string;
  /**
   * Optional custom CSS class for styling
   */
  className?: string;
  /**
   * Show detailed breakdown (total, available, sold percentage)
   * @default false
   */
  showDetails?: boolean;
  /**
   * Custom polling interval in milliseconds (default: 5000ms)
   * Set to 0 to disable auto-polling
   */
  pollInterval?: number;
}

/**
 * Component to display real-time ticket availability
 *
 * Shows:
 * - Availability status message
 * - Visual progress bar (if not sold out)
 * - Warning states (low stock, almost sold out)
 * - Optional detailed breakdown
 *
 * @example
 * ```typescript
 * // Basic usage
 * <TicketAvailabilityDisplay eventId="evt_123" />
 *
 * // With details and custom polling
 * <TicketAvailabilityDisplay
 *   eventId="evt_123"
 *   showDetails
 *   pollInterval={3000}
 *   className="p-4 rounded-lg bg-blue-50"
 * />
 * ```
 */
export function TicketAvailabilityDisplay({
  eventId,
  className = "",
  showDetails = false,
  pollInterval = 5000,
}: TicketAvailabilityDisplayProps) {
  const { data, isLoading, error } = useTicketAvailability(eventId, {
    pollInterval,
  });

  if (error) {
    return (
      <div className={`text-sm text-red-600 ${className}`}>
        Unable to load ticket availability
      </div>
    );
  }

  if (isLoading || !data) {
    return (
      <div className={`text-sm text-gray-500 animate-pulse ${className}`}>
        Loading availability...
      </div>
    );
  }

  const statusMessage = getAvailabilityStatus(data);
  const percentageAvailable = 100 - data.percentageSold;

  // Determine status color based on availability
  const getStatusColor = () => {
    if (data.isSoldOut) return "text-red-600";
    if (data.availableTickets <= 5) return "text-orange-600";
    if (data.percentageSold > 75) return "text-yellow-600";
    return "text-green-600";
  };

  const getProgressBarColor = () => {
    if (data.isSoldOut) return "bg-red-500";
    if (data.availableTickets <= 5) return "bg-orange-500";
    if (data.percentageSold > 75) return "bg-yellow-500";
    return "bg-green-500";
  };

  return (
    <div className={`flex flex-col gap-3 ${className}`}>
      {/* Status Message */}
      <div className="flex items-center justify-between">
        <p className={`text-sm font-medium ${getStatusColor()}`}>
          {statusMessage}
        </p>
        {data.isUsingSSE && (
          <span className="text-xs text-gray-400 flex items-center gap-1">
            <span className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
            Live
          </span>
        )}
      </div>

      {/* Progress Bar */}
      {!data.isSoldOut && (
        <div className="w-full bg-gray-200 rounded-full h-2.5 overflow-hidden">
          <div
            className={`h-full ${getProgressBarColor()} transition-all duration-300`}
            style={{ width: `${percentageAvailable}%` }}
          />
        </div>
      )}

      {/* Detailed Breakdown */}
      {showDetails && (
        <div className="text-xs text-gray-600 space-y-1 pt-2 border-t border-gray-200">
          <div className="flex justify-between">
            <span>Available:</span>
            <span className="font-semibold">{data.availableTickets}</span>
          </div>
          <div className="flex justify-between">
            <span>Minted:</span>
            <span className="font-semibold">{data.mintedTickets}</span>
          </div>
          <div className="flex justify-between">
            <span>Total:</span>
            <span className="font-semibold">{data.totalTickets}</span>
          </div>
          <div className="flex justify-between">
            <span>Sold:</span>
            <span className="font-semibold">{data.percentageSold}%</span>
          </div>
        </div>
      )}

      {/* Sold Out Notice */}
      {data.isSoldOut && (
        <div className="text-xs font-medium text-red-600 bg-red-50 p-2 rounded">
          This event is no longer accepting new ticket purchases.
        </div>
      )}

      {/* Low Stock Warning */}
      {!data.isSoldOut && data.availableTickets <= 5 && (
        <div className="text-xs font-medium text-orange-600 bg-orange-50 p-2 rounded">
          ⚠️ Very limited tickets remaining. Secure yours now!
        </div>
      )}
    </div>
  );
}
