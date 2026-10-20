import { render, screen, waitFor } from "@testing-library/react";
import { vi, describe, it, expect, beforeEach } from "vitest";
import EventLocationMap, { getCategoryIcon } from "@/components/events/event-location-map";

vi.mock("react-leaflet", () => ({
  MapContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="map-container">{children}</div>
  ),
  TileLayer: () => <div data-testid="tile-layer" />,
  Marker: ({ children, icon }: { children?: React.ReactNode; icon?: unknown }) => (
    <div data-testid="marker" data-icon={JSON.stringify(icon)}>
      {children}
    </div>
  ),
  Popup: ({ children }: { children?: React.ReactNode }) => (
    <div data-testid="popup">{children}</div>
  ),
  useMap: () => ({ setView: vi.fn(), getZoom: () => 13 }),
}));

vi.mock("leaflet", () => {
  const Icon = class {
    constructor() {}
  };
  (Icon as unknown as { Default: unknown }).Default = {
    prototype: {},
    mergeOptions: vi.fn(),
  };
  const divIcon = vi.fn((opts: { html: string; className: string }) => opts);
  return {
    default: {
      Icon,
      divIcon,
    },
    Icon,
    divIcon,
  };
});

const mockFetch = vi.fn();
global.fetch = mockFetch;

beforeEach(() => {
  global.fetch = mockFetch;
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

  it("generates category-specific marker icon styling", () => {
    const musicIcon = getCategoryIcon("Music");
    expect((musicIcon as unknown as { html: string }).html).toContain("#8B5CF6");
    expect((musicIcon as unknown as { html: string }).html).toContain('data-category="music"');

    const techIcon = getCategoryIcon("Tech");
    expect((techIcon as unknown as { html: string }).html).toContain("#2563EB");
    expect((techIcon as unknown as { html: string }).html).toContain('data-category="tech"');

    const workshopIcon = getCategoryIcon("Workshop");
    expect((workshopIcon as unknown as { html: string }).html).toContain("#F59E0B");
    expect((workshopIcon as unknown as { html: string }).html).toContain('data-category="workshop"');

    const sportsIcon = getCategoryIcon("Sports");
    expect((sportsIcon as unknown as { html: string }).html).toContain("#10B981");
    expect((sportsIcon as unknown as { html: string }).html).toContain('data-category="sports"');

    const defaultIcon = getCategoryIcon("Other");
    expect((defaultIcon as unknown as { html: string }).html).toContain("#6366F1");
  });

  it("renders popup with thumbnail card, category badge, title, location, and price", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => [{ lat: "40.7128", lon: "-74.0060" }],
    });

    render(
      <EventLocationMap
        location="New York, NY"
        category="Music"
        title="Jazz Under the Stars"
        price={45}
        imageUrl="https://example.com/jazz.jpg"
      />
    );

    await waitFor(() =>
      expect(screen.getByTestId("map-container")).toBeInTheDocument()
    );

    expect(screen.getByTestId("marker-popup-content")).toBeInTheDocument();
    expect(screen.getByTestId("popup-category")).toHaveTextContent("Music");
    expect(screen.getByTestId("popup-title")).toHaveTextContent("Jazz Under the Stars");
    expect(screen.getByTestId("popup-location")).toHaveTextContent("New York, NY");
    expect(screen.getByTestId("popup-price")).toHaveTextContent("$45");
    expect(screen.getByAltText("Jazz Under the Stars")).toHaveAttribute(
      "src",
      "https://example.com/jazz.jpg"
    );
  });
});
