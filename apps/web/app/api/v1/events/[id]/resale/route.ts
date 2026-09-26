import { NextRequest, NextResponse } from "next/server";

export interface ResaleListingItem {
  id: string;
  sellerName: string;
  sellerAvatar: string;
  sellerAddress?: string;
  price: number;
  originalPrice: number;
  quantity: number;
  listedAt: string;
}

// Mock database store for resale listings grouped by event ID
const mockResaleListingsMap: Record<string, ResaleListingItem[]> = {
  "1": [
    {
      id: "rsl-001",
      sellerName: "Alex M.",
      sellerAvatar: "/images/pfp.png",
      sellerAddress: "GALEX1234567890",
      price: 75,
      originalPrice: 49,
      quantity: 2,
      listedAt: "2 hours ago",
    },
    {
      id: "rsl-002",
      sellerName: "Jordan K.",
      sellerAvatar: "/images/pfp.png",
      sellerAddress: "GJORDAN1234567890",
      price: 60,
      originalPrice: 49,
      quantity: 1,
      listedAt: "5 hours ago",
    },
    {
      id: "rsl-003",
      sellerName: "Sam T.",
      sellerAvatar: "/images/pfp.png",
      sellerAddress: "GSAM1234567890",
      price: 55,
      originalPrice: 49,
      quantity: 1,
      listedAt: "1 day ago",
    },
  ],
};

/**
 * GET /api/v1/events/:id/resale
 *
 * Returns all active resale ticket listings for a specific event.
 */
export async function GET(
  req: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id: eventId } = await params;

    if (!eventId) {
      return NextResponse.json(
        { message: "Missing event ID" },
        { status: 400 }
      );
    }

    const listings = mockResaleListingsMap[eventId] || [
      {
        id: `rsl-default-${eventId}-1`,
        sellerName: "Alex M.",
        sellerAvatar: "/images/pfp.png",
        sellerAddress: "GALEX1234567890",
        price: 75,
        originalPrice: 49,
        quantity: 1,
        listedAt: "1 hour ago",
      },
    ];

    return NextResponse.json(
      {
        success: true,
        eventId,
        count: listings.length,
        data: listings,
      },
      { status: 200 }
    );
  } catch (err: any) {
    return NextResponse.json(
      { message: err.message || "Internal server error" },
      { status: 500 }
    );
  }
}
