import { buildMetadata } from "@/components/layout/seo";
import { MapDiscoveryPage } from "@/components/events/map-discovery-page";

export const metadata = buildMetadata({
  title: "Explore Events on the Map",
  description:
    "Discover events happening near you with Agora's interactive map. Browse by location and find your next experience.",
  path: "/discover/map",
});

export default function MapDiscoveryRoute() {
  return <MapDiscoveryPage />;
}