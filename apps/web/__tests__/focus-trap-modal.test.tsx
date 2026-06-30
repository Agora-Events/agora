import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { TicketModal } from "@/components/events/TicketModal";

vi.mock("next/image", () => ({
  default: ({ src, alt }: { src: string; alt: string }) => (
    // eslint-disable-next-line @next/next/no-img-element
    <img src={src} alt={alt} />
  ),
}));

vi.mock("qrcode.react", () => ({
  QRCodeSVG: ({ value }: { value: string }) => <svg data-testid="qr" data-value={value} />,
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const mockEvent = {
  id: 1,
  title: "Test Event",
  price: "10.00",
  location: "Online",
  date: "Sat, 27 Jun",
};

describe("TicketModal focus trap", () => {
  it("has role=dialog and aria-modal when open", () => {
    render(
      <TicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        initialQuantity={1}
      />
    );
    const dialog = screen.getByRole("dialog");
    expect(dialog).toBeInTheDocument();
    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(dialog).toHaveAttribute("aria-labelledby", "ticket-modal-title");
  });

  it("Tab key does not move focus outside the modal", () => {
    const outsideBtn = document.createElement("button");
    outsideBtn.textContent = "Outside";
    document.body.appendChild(outsideBtn);

    render(
      <TicketModal
        isOpen={true}
        onClose={vi.fn()}
        event={mockEvent}
        initialQuantity={1}
      />
    );

    const dialog = screen.getByRole("dialog");
    const focusableEls = dialog.querySelectorAll<HTMLElement>(
      'a[href],button:not([disabled]),input:not([disabled]),[tabindex]:not([tabindex="-1"])'
    );

    expect(focusableEls.length).toBeGreaterThan(0);

    const lastEl = focusableEls[focusableEls.length - 1];
    lastEl.focus();

    fireEvent.keyDown(document, { key: "Tab", shiftKey: false });

    const activeEl = document.activeElement;
    expect(dialog.contains(activeEl)).toBe(true);
    expect(activeEl).not.toBe(outsideBtn);

    document.body.removeChild(outsideBtn);
  });

  it("calls onClose when Escape is pressed", () => {
    const onClose = vi.fn();
    render(
      <TicketModal
        isOpen={true}
        onClose={onClose}
        event={mockEvent}
        initialQuantity={1}
      />
    );
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("does not render when isOpen is false", () => {
    render(
      <TicketModal
        isOpen={false}
        onClose={vi.fn()}
        event={mockEvent}
        initialQuantity={1}
      />
    );
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});
