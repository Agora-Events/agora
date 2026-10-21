import { NextResponse } from "next/server";

const BACKEND_URL =
  process.env.BACKEND_URL ??
  process.env.NEXT_PUBLIC_API_URL ??
  "http://localhost:3001";

interface BackendListing {
  payment_id: string;
  seller_wallet: string;
  price_stroops: number | string;
  created_at: string;
}

function relativeListedAt(createdAt: string): string {
  const elapsedMs = Math.max(0, Date.now() - new Date(createdAt).getTime());
  const hours = Math.floor(elapsedMs / 3_600_000);
  if (hours < 1) return "less than an hour ago";
  if (hours < 24) return `${hours} hour${hours === 1 ? "" : "s"} ago`;
  const days = Math.floor(hours / 24);
  return `${days} day${days === 1 ? "" : "s"} ago`;
}

export async function GET(
  { params }: { params: Promise<{ id: string }> },
) {
  const { id: eventId } = await params;
  if (!eventId) {
    return NextResponse.json({ message: "Missing event ID" }, { status: 400 });
  }

  try {
    const backendUrl = new URL("/api/v1/marketplace/listings", BACKEND_URL);
    backendUrl.searchParams.set("event_id", eventId);
    backendUrl.searchParams.set("status", "active");
    const response = await fetch(backendUrl, { cache: "no-store" });
    const result = await response.json().catch(() => ({}));
    if (!response.ok) {
      return NextResponse.json(
        { message: result.message || "Could not load resale listings" },
        { status: response.status },
      );
    }

    const listings = (Array.isArray(result.data) ? result.data : []).map(
      (listing: BackendListing) => {
        const sellerAddress = listing.seller_wallet;
        const priceStroops = String(listing.price_stroops);
        return {
          id: listing.payment_id,
          sellerName: `${sellerAddress.slice(0, 5)}…${sellerAddress.slice(-4)}`,
          sellerAvatar: "/images/pfp.png",
          sellerAddress,
          price: Number(priceStroops) / 10_000_000,
          priceStroops,
          quantity: 1,
          listedAt: relativeListedAt(listing.created_at),
        };
      },
    );

    return NextResponse.json({ success: true, data: listings });
  } catch (error) {
    console.error("Resale listing proxy failed:", error);
    return NextResponse.json(
      { message: "Marketplace is temporarily unavailable" },
      { status: 502 },
    );
  }
}