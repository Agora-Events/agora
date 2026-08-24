import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { vi, describe, it, expect, beforeEach } from "vitest";
import EventMap from "@/components/events/event-map";

vi.mock("react-leaflet", () => ({
  MapContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="map-container">{children}</div>
  ),
  TileLayer: () => <div data-testid="tile-layer" />,
  Marker: ({ children }: { children?: React.ReactNode }) => (
    <div data-testid="marker">{children}</div>
  ),
  Popup: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="popup">{children}</div>
  ),
  useMap: () => ({ setView: vi.fn(), fitBounds: vi.fn(), getZoom: () => 13 }),
}));

vi.mock("leaflet", () => {
  const iconDefault = {
    prototype: {} as Record<string, unknown>,
    mergeOptions: vi.fn(),
  };
  const IconCtor = vi.fn(function (
    this: unknown,
    opts: Record<string, unknown>
  ) {
    return { opts };
  }) as unknown as {
    Default: typeof iconDefault;
    mergeOptions: ReturnType<typeof vi.fn>;
  };
  IconCtor.Default = iconDefault;
  IconCtor.mergeOptions = vi.fn();
  const mockIcon = vi.fn((_opts: Record<string, unknown>) => ({ _opts }));
  return {
    default: {
      Icon: IconCtor,
      icon: mockIcon,
      latLngBounds: vi.fn(() => ({})),
    },
    Icon: IconCtor,
  };
});

const mockFetch = vi.fn();
global.fetch = mockFetch;

const sampleEvents = {
  popularEvents: [
    {
      id: "evt-1",
      title: "Stellar Builders Summit",
      date: "2026-09-01",
      location: "Lagos, Nigeria",
      latitude: 6.5244,
      longitude: 3.3792,
      price: "Free",
      imageUrl: "/images/event1.png",
      category: "Technology",
    },
    {
      id: "evt-2",
      title: "Web3 Hackathon",
      date: "2026-09-05",
      location: "Accra, Ghana",
      latitude: 5.6037,
      longitude: -0.187,
      price: "50",
      imageUrl: "/images/event2.png",
      category: "Web3",
    },
  ],
};

describe("EventMap", () => {
  beforeEach(() => {
    mockFetch.mockReset();
  });

  it("shows a loading state while fetching events", () => {
    mockFetch.mockReturnValue(new Promise(() => {}));
    render(<EventMap />);
    expect(screen.getByRole("status")).toBeInTheDocument();
    expect(screen.getByLabelText("Loading map")).toBeInTheDocument();
  });

  it("renders the map with event markers after fetch", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => sampleEvents,
    });
    render(<EventMap />);

    await waitFor(() =>
      expect(screen.getByTestId("map-container")).toBeInTheDocument()
    );
    const markers = screen.getAllByTestId("marker");
    expect(markers).toHaveLength(2);
  });

  it("shows event details in popups", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => sampleEvents,
    });
    render(<EventMap />);

    await waitFor(() =>
      expect(screen.getByTestId("map-container")).toBeInTheDocument()
    );
    // Markers render popup content (mocked Marker renders children)
    expect(screen.getByText("Stellar Builders Summit")).toBeInTheDocument();
    expect(screen.getByText("Web3 Hackathon")).toBeInTheDocument();
    expect(screen.getByText("Lagos, Nigeria")).toBeInTheDocument();
  });

  it("shows an error state when fetching fails", async () => {
    mockFetch.mockResolvedValueOnce({ ok: false, json: async () => ({}) });
    render(<EventMap />);

    await waitFor(() => expect(screen.getByRole("alert")).toBeInTheDocument());
    expect(screen.getByText(/failed to load map data/i)).toBeInTheDocument();
  });

  it("shows error when fetch throws", async () => {
    mockFetch.mockRejectedValueOnce(new Error("network error"));
    render(<EventMap />);

    await waitFor(() => expect(screen.getByRole("alert")).toBeInTheDocument());
  });

  it("shows empty state when no events are returned", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ popularEvents: [] }),
    });
    render(<EventMap />);

    await waitFor(() =>
      expect(screen.getByTestId("map-empty")).toBeInTheDocument()
    );
    expect(screen.getByText(/no events to display/i)).toBeInTheDocument();
  });

  it("renders initial events without fetching", async () => {
    render(<EventMap initialEvents={sampleEvents.popularEvents as never} />);

    await waitFor(() =>
      expect(screen.getByTestId("map-container")).toBeInTheDocument()
    );
    expect(mockFetch).not.toHaveBeenCalled();
  });

  it("retries the fetch when the retry button is clicked", async () => {
    mockFetch.mockResolvedValueOnce({ ok: false, json: async () => ({}) });
    render(<EventMap />);

    await waitFor(() => expect(screen.getByRole("alert")).toBeInTheDocument());

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => sampleEvents,
    });
    fireEvent.click(screen.getByRole("button", { name: /retry/i }));

    await waitFor(() =>
      expect(screen.getByTestId("map-container")).toBeInTheDocument()
    );
  });
});