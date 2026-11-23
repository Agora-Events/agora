"use client";

import useSWR from "swr";

const numberFormatter = new Intl.NumberFormat();

async function fetchFollowerCount(url: string): Promise<number> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error("Could not load follower count");
  }

  const result = await response.json();
  const count = result.data?.followers_count;
  if (!result.success || typeof count !== "number") {
    throw new Error("Invalid follower count response");
  }
  return count;
}

export function OrganizerFollowerCount({ organizerId }: { organizerId: string }) {
  const { data: count, error, isLoading } = useSWR(
    `/api/v1/organizers/${encodeURIComponent(organizerId)}/followers/count`,
    fetchFollowerCount,
  );

  if (error) return null;

  if (isLoading) {
    return (
      <span
        role="status"
        aria-label="Loading follower count"
        className="h-4 w-24 shrink-0 animate-pulse rounded bg-gray-200 motion-reduce:animate-none"
      />
    );
  }

  if (count === undefined) return null;

  return (
    <span className="shrink-0 text-sm text-gray-600">
      {numberFormatter.format(count)} {count === 1 ? "follower" : "followers"}
    </span>
  );
}
