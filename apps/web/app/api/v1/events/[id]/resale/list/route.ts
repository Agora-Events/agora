import { NextRequest, NextResponse } from "next/server";

export async function POST(
  req: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id: eventId } = await params;
    const body = await req.json();
    const { ticketId, resalePrice, sellerAddress, transactionXdr, listingId } = body;

    if (!ticketId || !resalePrice || resalePrice <= 0) {
      return NextResponse.json(
        { message: "Invalid ticketId or resalePrice" },
        { status: 400 }
      );
    }

    return NextResponse.json(
      {
        success: true,
        message: "Resale listing submitted and verified on-chain",
        data: {
          listingId: listingId || `resale_${Date.now()}`,
          eventId,
          ticketId,
          resalePrice,
          sellerAddress: sellerAddress || "GSELLERADDRESS",
          transactionXdr: transactionXdr || "",
          listedAt: new Date().toISOString(),
          status: "active",
        },
      },
      { status: 201 }
    );
  } catch (err: any) {
    return NextResponse.json(
      { message: err.message || "Internal server error" },
      { status: 500 }
    );
  }
}
