import { render, screen, waitForElementToBeRemoved } from "@testing-library/react";
import { SWRConfig } from "swr";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OrganizerFollowerCount } from "@/components/profile/organizer-follower-count";

function renderFollowerCount() {
  return render(
    <SWRConfig value={{ provider: () => new Map(), shouldRetryOnError: false }}>
      <OrganizerFollowerCount organizerId="organizer-1" />
    </SWRConfig>,
  );
}

afterEach(() => vi.unstubAllGlobals());

describe("OrganizerFollowerCount", () => {
  it("shows a skeleton and then the formatted follower count", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      Response.json({ success: true, data: { followers_count: 1204 } }),
    );
    vi.stubGlobal("fetch", fetchMock);

    renderFollowerCount();

    expect(screen.getByRole("status", { name: "Loading follower count" }))
      .toBeInTheDocument();
    expect(await screen.findByText(`${new Intl.NumberFormat().format(1204)} followers`))
      .toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/v1/organizers/organizer-1/followers/count",
    );
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("hides the follower count when the request fails", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(
      Response.json({ message: "Unavailable" }, { status: 503 }),
    ));

    const { container } = renderFollowerCount();

    await waitForElementToBeRemoved(screen.getByRole("status"));
    expect(container).toBeEmptyDOMElement();
  });
});
