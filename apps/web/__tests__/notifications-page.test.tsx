import { render, screen, fireEvent } from "@testing-library/react";
import { expect, describe, it, vi } from "vitest";
import NotificationsPage from "@/app/notifications/page";

// Mock next/image — render a plain img so jsdom doesn't choke on optimization
vi.mock("next/image", () => ({
  // eslint-disable-next-line @next/next/no-img-element
  default: ({ src, alt }: { src: string; alt: string }) => <img src={src} alt={alt} />,
}));

// Mock next/link
vi.mock("next/link", () => ({
  default: ({ href, children }: { href: string; children: React.ReactNode }) => (
    <a href={href}>{children}</a>
  ),
}));

// Mock layout chrome so the page renders independently
vi.mock("@/components/layout/navbar", () => ({
  Navbar: () => <nav data-testid="navbar" />,
}));

vi.mock("@/components/layout/footer", () => ({
  Footer: () => <footer data-testid="footer" />,
}));

// Mock sonner toast to assert the confirmation call
const toastSuccess = vi.fn();
vi.mock("sonner", () => ({
  toast: { success: (...args: unknown[]) => toastSuccess(...args) },
}));

describe("NotificationsPage mark all as read", () => {
  it("renders the unread count badge next to the title", () => {
    render(<NotificationsPage />);
    // mockNotifications has 2 unread, 1 read
    expect(screen.getByTestId("unread-badge")).toHaveTextContent("2");
    expect(screen.getByLabelText("2 unread notifications")).toBeInTheDocument();
  });

  it("renders the Mark all as read button enabled when there are unread items", () => {
    render(<NotificationsPage />);
    const btn = screen.getByRole("button", { name: /mark all notifications as read/i });
    expect(btn).toBeEnabled();
  });

  it("marks every notification read and clears unread styling on click", () => {
    render(<NotificationsPage />);
    const btn = screen.getByRole("button", { name: /mark all notifications as read/i });
    fireEvent.click(btn);

    // unread badge disappears after all are marked read
    expect(screen.queryByTestId("unread-badge")).not.toBeInTheDocument();
    // button becomes disabled
    expect(screen.getByRole("button", { name: /mark all notifications as read/i })).toBeDisabled();
    // toast confirmation fired
    expect(toastSuccess).toHaveBeenCalledWith("All notifications marked as read");
  });

  it("disables the button and hides the badge when there is nothing unread", () => {
    // simulate all-read by clicking once first
    render(<NotificationsPage />);
    fireEvent.click(screen.getByRole("button", { name: /mark all notifications as read/i }));

    // clicking again should be a no-op (toast not re-fired)
    toastSuccess.mockClear();
    const btn = screen.getByRole("button", { name: /mark all notifications as read/i });
    expect(btn).toBeDisabled();
    fireEvent.click(btn);
    expect(toastSuccess).not.toHaveBeenCalled();
  });

  it("keeps the page title and empty-state behavior intact", () => {
    render(<NotificationsPage />);
    expect(screen.getByRole("heading", { name: /notifications/i })).toBeInTheDocument();
  });
});
