import type { MetadataRoute } from "next";

export default function manifest(): MetadataRoute.Manifest {
  return {
    name: "Agora - Decentralized Event Ticketing Platform",
    short_name: "Agora",
    description:
      "Discover, buy, and resell event tickets on the Stellar blockchain with low fees and instant settlement.",
    start_url: "/",
    display: "standalone",
    background_color: "#0A0A0A",
    theme_color: "#FF4F11",
    icons: [
      {
        src: "/icon.png",
        sizes: "32x32",
        type: "image/png",
      },
      {
        src: "/apple-icon.png",
        sizes: "180x180",
        type: "image/png",
      },
      {
        src: "/icon-192.png",
        sizes: "192x192",
        type: "image/png",
      },
      {
        src: "/icon-512.png",
        sizes: "512x512",
        type: "image/png",
      },
    ],
  };
}
