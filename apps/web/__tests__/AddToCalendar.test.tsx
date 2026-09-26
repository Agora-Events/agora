import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { AddToCalendar } from "@/components/events/add-to-calendar";
import { buildGoogleCalendarUrl } from "@/utils/calendar";

vi.mock("next/image", () => ({
  default: ({ alt }: { alt: string }) => (
    // eslint-disable-next-line @next/next/no-img-element
    <img alt={alt} />
  ),
}));

const event = {
  id: 42,
  title: "Stellar Asado",
  description: "Builder kickoff",
  location: "Buenos Aires",
  startsAt: "2026-11-17T18:00:00Z",
};

describe("AddToCalendar", () => {
  const createObjectURL = vi.fn(() => "blob:mock-ics");
  const revokeObjectURL = vi.fn();
  let clickSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.clearAllMocks();
    Object.defineProperty(URL, "createObjectURL", {
      configurable: true,
      writable: true,
      value: createObjectURL,
    });
    Object.defineProperty(URL, "revokeObjectURL", {
      configurable: true,
      writable: true,
      value: revokeObjectURL,
    });
    clickSpy = vi
      .spyOn(HTMLAnchorElement.prototype, "click")
      .mockImplementation(() => undefined);
  });

  afterEach(() => {
    clickSpy.mockRestore();
  });

  it("renders both calendar options when opened", () => {
    render(<AddToCalendar event={event} />);

    fireEvent.click(screen.getByRole("button", { name: /add to calendar/i }));

    expect(screen.getByRole("link", { name: /google calendar/i })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /apple \/ outlook \(\.ics\)/i })
    ).toBeInTheDocument();
  });

  it("builds a Google Calendar link with title, dates, and location", () => {
    render(<AddToCalendar event={event} />);

    fireEvent.click(screen.getByRole("button", { name: /add to calendar/i }));

    const link = screen.getByRole("link", { name: /google calendar/i });
    const expected = buildGoogleCalendarUrl(event);
    expect(link).toHaveAttribute("href", expected);
    expect(link.getAttribute("href")).toContain("text=Stellar+Asado");
    expect(link.getAttribute("href")).toContain("location=Buenos+Aires");
    expect(link.getAttribute("href")).toContain(
      "dates=20261117T180000Z%2F20261117T200000Z"
    );
  });

  it("creates an .ics download when the ics option is clicked", async () => {
    render(<AddToCalendar event={event} />);

    fireEvent.click(screen.getByRole("button", { name: /add to calendar/i }));
    fireEvent.click(
      screen.getByRole("button", { name: /apple \/ outlook \(\.ics\)/i })
    );

    await waitFor(() => {
      expect(createObjectURL).toHaveBeenCalled();
      expect(clickSpy).toHaveBeenCalled();
      expect(revokeObjectURL).toHaveBeenCalledWith("blob:mock-ics");
    });

    const blobArg = createObjectURL.mock.calls[0]?.[0] as Blob;
    expect(blobArg).toBeInstanceOf(Blob);
    expect(blobArg.type).toContain("text/calendar");
  });

  it("applies a custom className on the root container", () => {
    const { container } = render(
      <AddToCalendar event={event} className="custom-calendar-class" />
    );

    expect(container.firstChild).toHaveClass("custom-calendar-class");
  });
});
