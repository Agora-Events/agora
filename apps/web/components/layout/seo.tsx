import type { Metadata } from "next";

const BASE_URL = "https://agora.events";
const DEFAULT_IMAGE = "/og-image.png";

interface SEOProps {
  title: string;
  description: string;
  image?: string;
  path?: string;
}

export function buildMetadata({ title, description, image, path }: SEOProps): Metadata {
  const url = path ? `${BASE_URL}${path}` : BASE_URL;
  const ogImage = image ?? DEFAULT_IMAGE;

  return {
    title,
    description,
    openGraph: {
      title,
      description,
      url,
      images: [{ url: ogImage, width: 1200, height: 630, alt: title }],
      type: "website",
    },
  };
}
