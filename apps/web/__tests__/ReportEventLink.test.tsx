import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { ReportEventLink } from "@/components/events/ReportEventLink";
import { SUPPORT_EMAIL } from "@/lib/constants";

vi.mock("next-intl", () => ({
  useTranslations: () => (key: string) => (key === "reportEvent" ? "Report this event" : key),
}));

describe("ReportEventLink", () => {
  const pageUrl = "https://agora.events/events/42";

  beforeEach(() => {
    vi.clearAllMocks();
    Object.defineProperty(window, "location", {
      configurable: true,
      value: { href: pageUrl },
    });
  });

  it("renders the report event link", () => {
    render(<ReportEventLink eventId={42} />);
    const link = screen.getByRole("link", { name: /report this event/i });
    expect(link).toBeInTheDocument();
  });

  it("constructs mailto href with support email, subject including event id, and body with event url", () => {
    render(<ReportEventLink eventId={42} />);
    const link = screen.getByRole("link", { name: /report this event/i });
    
    const expectedSubject = encodeURIComponent("Report event 42");
    const expectedBody = encodeURIComponent(pageUrl);
    const expectedHref = `mailto:${SUPPORT_EMAIL}?subject=${expectedSubject}&body=${expectedBody}`;

    expect(link).toHaveAttribute("href", expectedHref);
  });
});
