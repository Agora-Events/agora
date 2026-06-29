import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { TicketAvailabilityDisplay } from "@/components/events/ticket-availability-display";
import {
  useTicketAvailability,
  getAvailabilityStatus,
} from "@/hooks/useTicketAvailability";

// Mock the useTicketAvailability hook
vi.mock("@/hooks/useTicketAvailability", async () => {
  const actual = await vi.importActual("@/hooks/useTicketAvailability");
  return {
    ...actual,
    useTicketAvailability: vi.fn(),
  };
});

describe("Ticket Availability", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  describe("getAvailabilityStatus", () => {
    it('returns "Sold Out" when no tickets available', () => {
      const data = {
        totalTickets: 100,
        mintedTickets: 100,
        availableTickets: 0,
        isSoldOut: true,
        percentageSold: 100,
        isUsingSSE: false,
      };

      expect(getAvailabilityStatus(data)).toBe("Sold Out");
    });

    it("returns low stock message when 5 or fewer tickets remain", () => {
      const data = {
        totalTickets: 100,
        mintedTickets: 96,
        availableTickets: 4,
        isSoldOut: false,
        percentageSold: 96,
        isUsingSSE: false,
      };

      expect(getAvailabilityStatus(data)).toBe("Only 4 left!");
    });

    it("returns almost sold out when more than 75% sold", () => {
      const data = {
        totalTickets: 100,
        mintedTickets: 80,
        availableTickets: 20,
        isSoldOut: false,
        percentageSold: 80,
        isUsingSSE: false,
      };

      expect(getAvailabilityStatus(data)).toBe("Almost Sold Out");
    });

    it("returns available tickets count for normal availability", () => {
      const data = {
        totalTickets: 100,
        mintedTickets: 30,
        availableTickets: 70,
        isSoldOut: false,
        percentageSold: 30,
        isUsingSSE: false,
      };

      expect(getAvailabilityStatus(data)).toBe("70 tickets available");
    });
  });

  describe("TicketAvailabilityDisplay", () => {
    it("shows loading state while fetching", () => {
      const mockUseTicketAvailability = useTicketAvailability as ReturnType<
        typeof vi.fn
      >;
      mockUseTicketAvailability.mockReturnValue({
        data: undefined,
        isLoading: true,
        error: null,
        refresh: vi.fn(),
        isUsingSSE: false,
      });

      render(<TicketAvailabilityDisplay eventId="evt_123" />);

      expect(screen.getByText(/loading availability/i)).toBeInTheDocument();
    });

    it("shows error message when fetch fails", () => {
      const mockUseTicketAvailability = useTicketAvailability as ReturnType<
        typeof vi.fn
      >;
      mockUseTicketAvailability.mockReturnValue({
        data: undefined,
        isLoading: false,
        error: new Error("Failed to fetch"),
        refresh: vi.fn(),
        isUsingSSE: false,
      });

      render(<TicketAvailabilityDisplay eventId="evt_123" />);

      expect(
        screen.getByText(/unable to load ticket availability/i),
      ).toBeInTheDocument();
    });

    it("displays availability status when data loads", () => {
      const mockUseTicketAvailability = useTicketAvailability as ReturnType<
        typeof vi.fn
      >;
      mockUseTicketAvailability.mockReturnValue({
        data: {
          totalTickets: 100,
          mintedTickets: 30,
          availableTickets: 70,
          isSoldOut: false,
          percentageSold: 30,
          isUsingSSE: false,
        },
        isLoading: false,
        error: null,
        refresh: vi.fn(),
        isUsingSSE: false,
      });

      render(<TicketAvailabilityDisplay eventId="evt_123" />);

      expect(screen.getByText(/70 tickets available/i)).toBeInTheDocument();
    });

    it("shows sold out message when event is sold out", () => {
      const mockUseTicketAvailability = useTicketAvailability as ReturnType<
        typeof vi.fn
      >;
      mockUseTicketAvailability.mockReturnValue({
        data: {
          totalTickets: 100,
          mintedTickets: 100,
          availableTickets: 0,
          isSoldOut: true,
          percentageSold: 100,
          isUsingSSE: false,
        },
        isLoading: false,
        error: null,
        refresh: vi.fn(),
        isUsingSSE: false,
      });

      render(<TicketAvailabilityDisplay eventId="evt_123" />);

      expect(screen.getByText(/sold out/i)).toBeInTheDocument();
      expect(
        screen.getByText(
          /this event is no longer accepting new ticket purchases/i,
        ),
      ).toBeInTheDocument();
    });

    it("shows low stock warning when only few tickets remain", () => {
      const mockUseTicketAvailability = useTicketAvailability as ReturnType<
        typeof vi.fn
      >;
      mockUseTicketAvailability.mockReturnValue({
        data: {
          totalTickets: 100,
          mintedTickets: 98,
          availableTickets: 2,
          isSoldOut: false,
          percentageSold: 98,
          isUsingSSE: false,
        },
        isLoading: false,
        error: null,
        refresh: vi.fn(),
        isUsingSSE: false,
      });

      render(<TicketAvailabilityDisplay eventId="evt_123" />);

      expect(
        screen.getByText(/very limited tickets remaining/i),
      ).toBeInTheDocument();
    });

    it("displays detailed breakdown when showDetails is true", () => {
      const mockUseTicketAvailability = useTicketAvailability as ReturnType<
        typeof vi.fn
      >;
      mockUseTicketAvailability.mockReturnValue({
        data: {
          totalTickets: 100,
          mintedTickets: 30,
          availableTickets: 70,
          isSoldOut: false,
          percentageSold: 30,
          isUsingSSE: false,
        },
        isLoading: false,
        error: null,
        refresh: vi.fn(),
        isUsingSSE: false,
      });

      render(
        <TicketAvailabilityDisplay eventId="evt_123" showDetails={true} />,
      );

      expect(screen.getByText(/available:/i)).toBeInTheDocument();
      expect(screen.getByText(/minted:/i)).toBeInTheDocument();
      expect(screen.getByText(/total:/i)).toBeInTheDocument();
      expect(screen.getByText(/sold:/i)).toBeInTheDocument();
    });

    it("shows live indicator when using SSE", () => {
      const mockUseTicketAvailability = useTicketAvailability as ReturnType<
        typeof vi.fn
      >;
      mockUseTicketAvailability.mockReturnValue({
        data: {
          totalTickets: 100,
          mintedTickets: 30,
          availableTickets: 70,
          isSoldOut: false,
          percentageSold: 30,
          isUsingSSE: true,
        },
        isLoading: false,
        error: null,
        refresh: vi.fn(),
        isUsingSSE: true,
      });

      render(<TicketAvailabilityDisplay eventId="evt_123" />);

      expect(screen.getByText(/live/i)).toBeInTheDocument();
    });

    it("displays progress bar with correct width based on availability", () => {
      const mockUseTicketAvailability = useTicketAvailability as ReturnType<
        typeof vi.fn
      >;
      mockUseTicketAvailability.mockReturnValue({
        data: {
          totalTickets: 100,
          mintedTickets: 50,
          availableTickets: 50,
          isSoldOut: false,
          percentageSold: 50,
          isUsingSSE: false,
        },
        isLoading: false,
        error: null,
        refresh: vi.fn(),
        isUsingSSE: false,
      });

      const { container } = render(
        <TicketAvailabilityDisplay eventId="evt_123" />,
      );

      // Progress bar width should be 50% available
      const progressBar = container.querySelector('[style*="width"]');
      expect(progressBar).toHaveStyle("width: 50%");
    });
  });
});
