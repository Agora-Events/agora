import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { ReportEventLink } from "@/components/events/ReportEventLink";

vi.mock("next-intl", () => ({
  useTranslations: () => (key: string) => (key === "reportEvent" ? "Report this event" : key),
}));

describe("ReportEventLink", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the report event button", () => {
    render(<ReportEventLink eventId={42} />);
    const button = screen.getByRole("button", { name: /report this event/i });
    expect(button).toBeInTheDocument();
  });

  it("opens the report modal with reason options", () => {
    render(<ReportEventLink eventId={42} />);
    fireEvent.click(screen.getByRole("button", { name: /report this event/i }));

    expect(screen.getByRole("dialog")).toBeInTheDocument();
    for (const reason of ["Spam", "Scam", "Inappropriate", "Cancelled"]) {
      expect(screen.getByLabelText(reason)).toBeInTheDocument();
    }
  });

  it("submits the selected reason to the flag endpoint", async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, json: async () => ({}) });
    vi.stubGlobal("fetch", fetchMock);

    render(<ReportEventLink eventId={42} />);
    fireEvent.click(screen.getByRole("button", { name: /report this event/i }));
    fireEvent.click(screen.getByLabelText("Spam"));
    fireEvent.click(screen.getByRole("button", { name: /submit report/i }));

    await vi.waitFor(() => expect(fetchMock).toHaveBeenCalled());
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/v1/events/42/flag",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ flagged: true, reason: "Spam" }),
      })
    );

    vi.unstubAllGlobals();
  });
});
