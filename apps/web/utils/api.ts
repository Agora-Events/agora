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

async function fetchDiscoverPayload(): Promise<DiscoverResponse> {
  const response = await fetch("/api/events/discover");
  if (!response.ok) {
    throw new Error("Unable to fetch discover data");
  }

  return response.json();
}

export async function fetchCategories(): Promise<DiscoverCategory[]> {
  const data = await fetchDiscoverPayload();
  return data.categories;
}

export async function fetchPopularEvents(): Promise<DiscoverEvent[]> {
  const data = await fetchDiscoverPayload();
  return data.popularEvents;
}

export async function fetchOrganizers(): Promise<DiscoverOrganizer[]> {
  const data = await fetchDiscoverPayload();
  return data.organizers;
}
