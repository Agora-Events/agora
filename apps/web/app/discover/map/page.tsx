import React from "react";
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import MapDiscoveryView from "@/components/events/map-discovery-view";

export const metadata = {
  title: "Discover Events on a Map",
  description:
    "Explore tech, crypto, wellness, and community events on an interactive map, clustering nearby events as you zoom in.",
};

export default function DiscoverMapPage() {
  return (
    <main
      id="main-content"
      tabIndex={-1}
      className="flex min-h-screen flex-col bg-base"
    >
      <Navbar />
      <div className="grow lg:h-[calc(100vh-91px)] lg:overflow-hidden">
        <MapDiscoveryView />
      </div>
      <Footer />
    </main>
  );
}
