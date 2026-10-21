import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ExportAttendeesButton } from "@/components/dashboard/ExportAttendeesButton";

describe("ExportAttendeesButton", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("renders with default label and icon", () => {
    render(<ExportAttendeesButton eventId="test-event-123" />);
    expect(
      screen.getByRole("button", { name: /Export Attendees \(CSV\)/i })
    ).toBeInTheDocument();
  });

  it("triggers file download on successful click", async () => {
    const csvContent = "owner_wallet,buyer_wallet,quantity,created_at\n0x1,0x2,1,2026-01-01T00:00:00Z";
    
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      headers: new Headers({
        "Content-Disposition": 'attachment; filename="attendees-test-event-123.csv"',
      }),
      blob: vi.fn().mockResolvedValue(new Blob([csvContent], { type: "text/csv" })),
    });

    const createObjectURLMock = vi.fn().mockReturnValue("blob:mock-url");
    const revokeObjectURLMock = vi.fn();
    window.URL.createObjectURL = createObjectURLMock;
    window.URL.revokeObjectURL = revokeObjectURLMock;

    // Prevent jsdom default navigation warning on anchor click
    const clickSpy = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});

    const onSuccessMock = vi.fn();

    render(
      <ExportAttendeesButton
        eventId="test-event-123"
        onSuccess={onSuccessMock}
      />
    );

    const button = screen.getByRole("button", { name: /Export Attendees \(CSV\)/i });
    fireEvent.click(button);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        "/api/v1/events/test-event-123/export-attendees",
        expect.objectContaining({ method: "GET", credentials: "include" })
      );
      expect(clickSpy).toHaveBeenCalledTimes(1);
      expect(onSuccessMock).toHaveBeenCalledTimes(1);
    });

    clickSpy.mockRestore();
  });

  it("handles and displays error message on download failure", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 403,
      text: vi.fn().mockResolvedValue("Only organizers can export attendees"),
    });

    const onErrorMock = vi.fn();

    render(
      <ExportAttendeesButton
        eventId="test-event-123"
        onError={onErrorMock}
      />
    );

    const button = screen.getByRole("button", { name: /Export Attendees \(CSV\)/i });
    fireEvent.click(button);

    await waitFor(() => {
      expect(
        screen.getByText("Only organizers can export attendees")
      ).toBeInTheDocument();
      expect(onErrorMock).toHaveBeenCalledTimes(1);
    });
  });
});
