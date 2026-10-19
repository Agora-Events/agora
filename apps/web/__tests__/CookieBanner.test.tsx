import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, beforeEach } from "vitest";
import { CookieBanner } from "@/components/layout/cookie-banner";
import { COOKIE_CONSENT_KEY } from "@/lib/constants";

describe("CookieBanner", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("renders when no consent choice is stored", async () => {
    render(<CookieBanner />);

    expect(await screen.findByRole("dialog", { name: "Cookie consent" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Accept" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Decline" })).toBeInTheDocument();
  });

  it("stores 'accepted' and hides the banner when Accept is clicked", async () => {
    render(<CookieBanner />);

    fireEvent.click(await screen.findByRole("button", { name: "Accept" }));

    expect(localStorage.getItem(COOKIE_CONSENT_KEY)).toBe("accepted");
    await waitFor(() => {
      expect(screen.queryByRole("dialog", { name: "Cookie consent" })).not.toBeInTheDocument();
    });
  });

  it("stores 'declined' and hides the banner when Decline is clicked", async () => {
    render(<CookieBanner />);

    fireEvent.click(await screen.findByRole("button", { name: "Decline" }));

    expect(localStorage.getItem(COOKIE_CONSENT_KEY)).toBe("declined");
    await waitFor(() => {
      expect(screen.queryByRole("dialog", { name: "Cookie consent" })).not.toBeInTheDocument();
    });
  });

  it.each(["accepted", "declined"])(
    "does not render when '%s' consent is already stored",
    async (choice) => {
      localStorage.setItem(COOKIE_CONSENT_KEY, choice);

      render(<CookieBanner />);

      await waitFor(() => {
        expect(screen.queryByRole("dialog", { name: "Cookie consent" })).not.toBeInTheDocument();
      });
    }
  );
});
