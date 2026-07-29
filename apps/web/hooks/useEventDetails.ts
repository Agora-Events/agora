"use client";

import useSWR from "swr";

/**
 * Typed API error that carries the HTTP status code.
 * Allows callers to distinguish 404 (not-found) from 5xx (server fault).
 */
export class EventApiError extends Error {
  constructor(
    public readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "EventApiError";
  }
}

const fetcher = async (url: string) => {
  const res = await fetch(url);
  if (!res.ok) {
    throw new EventApiError(
      res.status,
      `Failed to fetch event details (HTTP ${res.status})`,
    );
  }
  return res.json();
};

export function useEventDetails(id?: string) {
  const { data, error, isLoading, mutate } = useSWR(
    id ? `/api/v1/events/${id}` : null,
    fetcher,
    {
      // Only retry on non-404 errors — there is no point retrying a not-found.
      shouldRetryOnError: (err: unknown) =>
        !(err instanceof EventApiError && err.status === 404),
      errorRetryCount: 3,
    },
  );

  // Re-throw 5xx errors so they escape the React render and reach the nearest
  // error.tsx boundary.  404s are returned via isError so the page can call
  // notFound() itself — that behaviour is unchanged.
  if (error instanceof EventApiError && error.status >= 500) {
    throw error;
  }

  return {
    event: data,
    isLoading,
    isError: error,
    /** Re-invoke the fetch — pass as the retry callback to the error boundary. */
    retry: () => mutate(),
  };
}
