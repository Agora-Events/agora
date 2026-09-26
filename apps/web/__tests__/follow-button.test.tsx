import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, beforeEach, vi } from "vitest";
import { FollowButton } from "@/components/profile/follow-button";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const ORGANIZER_ID = "stellar-west-africa";
const STORAGE_KEY = "agora:followed-organizers";

describe("FollowButton", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("toggles between Follow and Following immediately and sets aria-pressed", async () => {
    render(<FollowButton organizerId={ORGANIZER_ID} />);

    const button = screen.getByRole("button", { name: /follow organizer/i });
    expect(button).toHaveAttribute("aria-pressed", "false");
    expect(button).toHaveTextContent("Follow");

    fireEvent.click(button);

    expect(button).toHaveAttribute("aria-pressed", "true");
    expect(button).toHaveTextContent("Following");

    await waitFor(() => {
      expect(JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}")[ORGANIZER_ID]).toBe(true);
    });
  });

  it("restores follow state from localStorage", async () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ [ORGANIZER_ID]: true }));

    render(<FollowButton organizerId={ORGANIZER_ID} />);

    await waitFor(() => {
      expect(screen.getByRole("button")).toHaveAttribute("aria-pressed", "true");
      expect(screen.getByRole("button")).toHaveTextContent("Following");
    });
  });

  it("rolls back the optimistic state when persistence fails", async () => {
    const setItem = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("quota exceeded");
    });

    render(<FollowButton organizerId={ORGANIZER_ID} />);

    fireEvent.click(screen.getByRole("button"));
    expect(screen.getByRole("button")).toHaveAttribute("aria-pressed", "true");

    await waitFor(() => {
      expect(screen.getByRole("button")).toHaveAttribute("aria-pressed", "false");
      expect(screen.getByRole("button")).toHaveTextContent("Follow");
    });

    setItem.mockRestore();
  });
});
