import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { vi, describe, it, expect, beforeEach } from "vitest";
import EventMap from "@/components/events/event-map";

// Mock supercluster
const mockGetClusters = vi.fn();
const mockGetClusterExpansionZoom = vi.fn();
const mockLoad = vi.fn();

vi.mock("supercluster", () => {
  const MockSupercluster = vi.fn(() => ({
    load: mockLoad,
    getClusters: mockGetClusters,
    getClusterExpansionZoom: mockGetClusterExpansionZoom,
  }));
  return { default: MockSupercluster };
});

vi.mock("react-leaflet", () => {
  const mockUseMap = () => ({
    setView: vi.fn(),
    fitBounds: vi.fn(),
    getZoom: () => 13,
    getBounds: () => ({
      getWest: () => -180,
      getSouth: () => -90,
      getEast: () => 180,
      getNorth: () => 90,
    }),
  });

  return {
    MapContainer: ({ children }: { children: React.ReactNode }) => (
      <div data-testid="map-container">{children}</div>
    ),
    TileLayer: () => <div data-testid="tile-layer" />,
    Marker: ({
      children,
      eventHandlers,
    }: {
      children?: React.ReactNode;
      eventHandlers?: Record<string, () => void>;
    }) => (
      <div
        data-testid="marker"
        onClick={eventHandlers?.click}
      >
        {children}
      </div>
    ),
    Popup: ({ children }: { children: React.ReactNode }) => (
      <div data-testid="popup">{children}</div>
    ),
    useMap: mockUseMap,
    useMapEvents: () => {},
  };
});

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
  const mockDivIcon = vi.fn((_opts: Record<string, unknown>) => ({ _opts, _isDivIcon: true }));
  return {
    default: {
      Icon: IconCtor,
      icon: mockIcon,
      DivIcon: mockDivIcon,
      divIcon: mockDivIcon,
      latLngBounds: vi.fn(() => ({
          getWest: () => -180,
          getSouth: () => -90,
          getEast: () => 180,
          getNorth: () => 90,
        })),
    },
    Icon: IconCtor,
    DivIcon: mockDivIcon,
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
    {
      id: "evt-3",
      title: "DeFi Workshop",
      date: "2026-09-10",
      location: "Nairobi, Kenya",
      latitude: -1.2921,
      longitude: 36.8219,
      price: "Free",
      imageUrl: "/images/event3.png",
      category: "DeFi",
    },
  ],
};

describe("EventMap", () => {
  beforeEach(() => {
    mockFetch.mockReset();
    mockGetClusters.mockReset();
    mockGetClusterExpansionZoom.mockReset();
    mockLoad.mockReset();
  });

  it("shows a loading state while fetching events", () => {
    mockFetch.mockReturnValue(new Promise(() => {}));
    render(<EventMap />);
    expect(screen.getByRole("status")).toBeInTheDocument();
    expect(screen.getByLabelText("Loading map")).toBeInTheDocument();
  });

  it("renders the map with clustered markers after fetch", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => sampleEvents,
    });

    // Simulate supercluster returning clusters:
    // first cluster is a group of 2 events, second is a single event
    mockGetClusters.mockReturnValueOnce([
      {
        type: "Feature",
        geometry: { type: "Point", coordinates: [3.3792, 6.5244] },
        properties: {
          cluster: true,
          cluster_id: 1,
          point_count: 2,
        },
      },
      {
        type: "Feature",
        geometry: { type: "Point", coordinates: [36.8219, -1.2921] },
        properties: {
          id: "evt-3",
          title: "DeFi Workshop",
          date: "2026-09-10",
          location: "Nairobi, Kenya",
          price: "Free",
          imageUrl: "/images/event3.png",
          category: "DeFi",
        },
      },
    ]);

    render(<EventMap />);

    await waitFor(() =>
      expect(screen.getByTestId("map-container")).toBeInTheDocument()
    );

    // Two markers: one cluster, one individual
    const markers = screen.getAllByTestId("marker");
    expect(markers).toHaveLength(2);
  });

  it("shows individual event details in popups", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => sampleEvents,
    });

    mockGetClusters.mockReturnValueOnce([
      {
        type: "Feature",
        geometry: { type: "Point", coordinates: [3.3792, 6.5244] },
        properties: {
          id: "evt-1",
          title: "Stellar Builders Summit",
          date: "2026-09-01",
          location: "Lagos, Nigeria",
          price: "Free",
          imageUrl: "/images/event1.png",
          category: "Technology",
        },
      },
      {
        type: "Feature",
        geometry: { type: "Point", coordinates: [-0.187, 5.6037] },
        properties: {
          id: "evt-2",
          title: "Web3 Hackathon",
          date: "2026-09-05",
          location: "Accra, Ghana",
          price: "50",
          imageUrl: "/images/event2.png",
          category: "Web3",
        },
      },
    ]);

    render(<EventMap />);

    await waitFor(() =>
      expect(screen.getByTestId("map-container")).toBeInTheDocument()
    );

    expect(screen.getByText("Stellar Builders Summit")).toBeInTheDocument();
    expect(screen.getByText("Web3 Hackathon")).toBeInTheDocument();
    expect(screen.getByText("Lagos, Nigeria")).toBeInTheDocument();
  });

  it("expands cluster on click", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => sampleEvents,
    });

    // First render: cluster with 2 events
    mockGetClusters.mockReturnValueOnce([
      {
        type: "Feature",
        geometry: { type: "Point", coordinates: [3.3792, 6.5244] },
        properties: {
          cluster: true,
          cluster_id: 1,
          point_count: 2,
        },
      },
    ]);

    mockGetClusterExpansionZoom.mockReturnValueOnce(15);

    render(<EventMap />);

    await waitFor(() =>
      expect(screen.getByTestId("map-container")).toBeInTheDocument()
    );

    // Click the cluster marker to expand
    const markers = screen.getAllByTestId("marker");
    expect(markers).toHaveLength(1);
    fireEvent.click(markers[0]);

    expect(mockGetClusterExpansionZoom).toHaveBeenCalledWith(1);
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
  });

  it("renders with initialEvents without fetching", () => {
    const initialEvents = sampleEvents.popularEvents.map((e) => ({
      ...e,
    }));

    mockGetClusters.mockReturnValueOnce([
      {
        type: "Feature",
        geometry: { type: "Point", coordinates: [3.3792, 6.5244] },
        properties: {
          id: "evt-1",
          title: "Stellar Builders Summit",
          date: "2026-09-01",
          location: "Lagos, Nigeria",
          price: "Free",
          imageUrl: "/images/event1.png",
          category: "Technology",
        },
      },
    ]);

    render(<EventMap initialEvents={initialEvents} />);
    expect(screen.getByTestId("map-container")).toBeInTheDocument();
  });

  it("builds supercluster index from event data", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => sampleEvents,
    });

    mockGetClusters.mockReturnValueOnce([]);

    render(<EventMap />);

    await waitFor(() =>
      expect(screen.getByTestId("map-container")).toBeInTheDocument()
    );

    // supercluster was instantiated and load was called with features
    expect(mockLoad).toHaveBeenCalled();
    const loadArgs = mockLoad.mock.calls[0][0];
    expect(loadArgs).toHaveLength(3); // 3 events with coordinates
  });
});