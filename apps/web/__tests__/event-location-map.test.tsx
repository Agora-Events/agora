import { render, screen, waitFor } from "@testing-library/react";
import { vi, describe, it, expect, beforeEach } from "vitest";
import EventLocationMap from "@/components/events/event-location-map";

vi.mock("react-leaflet", () => ({
  MapContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="map-container">{children}</div>
  ),
  TileLayer: () => <div data-testid="tile-layer" />,
  Marker: () => <div data-testid="marker" />,
  useMap: () => ({ setView: vi.fn(), getZoom: () => 13 }),
}));

vi.mock("leaflet", () => ({
  default: {
    Icon: {
      Default: { prototype: {}, mergeOptions: vi.fn() },
    },
    icon: vi.fn(),
  },
  Icon: class {
    constructor() {}
  },
}));

const mockFetch = vi.fn();
global.fetch = mockFetch;

beforeEach(() => {
  mockFetch.mockReset();
});

describe("EventLocationMap", () => {
  it("shows loader initially", () => {
    mockFetch.mockReturnValue(new Promise(() => {}));
    render(<EventLocationMap location="Lagos, Nigeria" />);
    expect(screen.getByRole("status")).toBeInTheDocument();
  });

  it("renders map after successful geocoding", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => [{ lat: "6.5244", lon: "3.3792" }],
    });
    render(<EventLocationMap location="Lagos, Nigeria" />);
    await waitFor(() =>
      expect(screen.getByTestId("map-container")).toBeInTheDocument()
    );
  });

  it("shows error message when geocoding fails", async () => {
    mockFetch.mockResolvedValueOnce({ ok: false, json: async () => [] });
    render(<EventLocationMap location="Unknown Place XYZ" />);
    await waitFor(() =>
      expect(screen.getByRole("alert")).toBeInTheDocument()
    );
  });

  it("shows error when fetch throws", async () => {
    mockFetch.mockRejectedValueOnce(new Error("network error"));
    render(<EventLocationMap location="Somewhere" />);
    await waitFor(() =>
      expect(screen.getByRole("alert")).toBeInTheDocument()
    );
  });
});
