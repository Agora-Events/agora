import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import type { ReactNode } from "react";
import { PopularEventsSection } from "@/components/events/popular-events-section";
import type { DiscoverEvent } from "@/utils/api";

vi.mock("next/image", () => ({
  default: ({ src, alt }: { src: string; alt: string }) => (
    // eslint-disable-next-line @next/next/no-img-element
    <img src={src} alt={alt} />
  ),
}));

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn() }),
}));

const mockEvents: DiscoverEvent[] = [
  {
    id: "1",
    title: "Later Tech Event",
    date: "Thu, 22 Mar, 1:00",
    location: "Lagos",
    price: "50",
    imageUrl: "/images/event1.png",
    category: "Tech",
    mintedTickets: 10,
  },
  {
    id: "2",
    title: "Soon Party Event",
    date: "Mon, 10 Feb, 9:00",
    location: "London",
    price: "Free",
    imageUrl: "/images/event2.png",
    category: "Party",
    mintedTickets: 80,
  },
  {
    id: "3",
    title: "Mid Tech Event",
    date: "Wed, 5 Mar, 10:00",
    location: "New York",
    price: "199",
    imageUrl: "/images/event3.png",
    category: "Tech",
    mintedTickets: 40,
  },
];

vi.mock("next/link", () => ({
  default: ({
    href,
    children,
    className,
  }: {
    href: string;
    children: ReactNode;
    className?: string;
  }) => (
    <a href={href} className={className}>
      {children}
    </a>
  ),
}));

vi.mock("@/utils/api", () => ({
  fetchPopularEvents: vi.fn(async () => ({
    events: mockEvents,
    meta: { total: mockEvents.length, page: 1, perPage: mockEvents.length },
  })),
}));

function titlesInOrder() {
  return screen
    .getAllByRole("link")
    .map((link) => link.textContent ?? "")
    .filter((text) =>
      mockEvents.some((event) => text.includes(event.title)),
    )
    .map((text) => mockEvents.find((event) => text.includes(event.title))!.title);
}

describe("PopularEventsSection sort", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("defaults to Date (soonest) and reorders for each sort option", async () => {
    render(<PopularEventsSection onError={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText("Soon Party Event")).toBeInTheDocument();
    });

    const sort = screen.getByLabelText("Sort events") as HTMLSelectElement;
    expect(sort).toHaveValue("date-soonest");
    expect(titlesInOrder()).toEqual([
      "Soon Party Event",
      "Mid Tech Event",
      "Later Tech Event",
    ]);

    fireEvent.change(sort, { target: { value: "price-asc" } });
    expect(titlesInOrder()).toEqual([
      "Soon Party Event",
      "Later Tech Event",
      "Mid Tech Event",
    ]);

    fireEvent.change(sort, { target: { value: "price-desc" } });
    expect(titlesInOrder()).toEqual([
      "Mid Tech Event",
      "Later Tech Event",
      "Soon Party Event",
    ]);

    fireEvent.change(sort, { target: { value: "popularity" } });
    expect(titlesInOrder()).toEqual([
      "Soon Party Event",
      "Mid Tech Event",
      "Later Tech Event",
    ]);
  });

  it("applies the chosen sort on top of the active category filter", async () => {
    render(
      <PopularEventsSection activeCategory="Tech" onError={vi.fn()} />,
    );

    await waitFor(() => {
      expect(screen.getByText("Later Tech Event")).toBeInTheDocument();
    });

    expect(screen.queryByText("Soon Party Event")).not.toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Sort events"), {
      target: { value: "price-desc" },
    });

    expect(titlesInOrder()).toEqual(["Mid Tech Event", "Later Tech Event"]);
  });
});
