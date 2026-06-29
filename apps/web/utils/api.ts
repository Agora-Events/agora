export type DiscoverCategory = {
  name: string;
  icon: string;
  color: string;
};

export type DiscoverEvent = {
  id: string;
  title: string;
  date: string;
  location: string;
  price: string;
  imageUrl: string;
  category: string;
};

export type DiscoverOrganizer = {
  id: string;
  title: string;
  description: string;
  image: string;
};

type DiscoverResponse = {
  categories: DiscoverCategory[];
  popularEvents: DiscoverEvent[];
  organizers: DiscoverOrganizer[];
};

// New: paginated events wrapper used by the frontend when the API provides pagination.
export type EventsPage = {
  events: DiscoverEvent[];
  meta: {
    total: number;
    page: number;
    perPage: number;
  };
};

// Accepts an optional AbortSignal so callers can cancel the request
// when a component unmounts, preventing state updates on unmounted components.
async function fetchDiscoverPayload(signal?: AbortSignal): Promise<DiscoverResponse> {
  const response = await fetch("/api/events/discover", { signal });
  if (!response.ok) {
    throw new Error("Unable to fetch discover data");
  }

  return response.json();
}

export async function fetchCategories(signal?: AbortSignal): Promise<DiscoverCategory[]> {
  const data = await fetchDiscoverPayload(signal);
  return data.categories;
}

// Fetch popular events. Supports an optional page parameter to request paginated responses
// If the backend still returns the old shape (all events), this function normalizes it to the EventsPage shape.
export async function fetchPopularEvents(page = 1, signal?: AbortSignal): Promise<EventsPage> {
  // Request the discover endpoint with a page query param. Backend may ignore if not supported.
  const url = `/api/events/discover?page=${page}`;
  const response = await fetch(url, { signal });
  if (!response.ok) {
    throw new Error("Unable to fetch popular events");
  }

  const data = await response.json();

  // Backward-compatible handling:
  // - If the endpoint returns the combined discover payload, use popularEvents inside it.
  // - If it returns an array directly, treat it as the full list with meta matching length.
  if (Array.isArray(data)) {
    const events = data as DiscoverEvent[];
    return {
      events,
      meta: { total: events.length, page: 1, perPage: events.length },
    };
  }

  if (data && Array.isArray(data.popularEvents)) {
    const events = data.popularEvents as DiscoverEvent[];
    // If the backend provides a meta object, use it; otherwise synthesize one.
    const meta = data.meta
      ? { total: data.meta.total ?? events.length, page: data.meta.page ?? page, perPage: data.meta.perPage ?? events.length }
      : { total: events.length, page, perPage: events.length };

    return { events, meta };
  }

  // Fallback: attempt to use the combined discover payload shape from fetchDiscoverPayload
  try {
    const combined = await fetchDiscoverPayload(signal);
    const events = combined.popularEvents || [];
    return { events, meta: { total: events.length, page, perPage: events.length } };
  } catch (err) {
    throw new Error("Unable to normalize popular events payload");
  }
}

export async function fetchOrganizers(signal?: AbortSignal): Promise<DiscoverOrganizer[]> {
  const data = await fetchDiscoverPayload(signal);
  return data.organizers;
}
