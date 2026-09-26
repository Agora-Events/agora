import { NextResponse } from "next/server";

export async function GET(
  request: Request,
  { params }: { params: Promise<{ id: string }> }
) {
  const { id } = await params;
  
  // Return dummy data for the mock dashboard.
  // In a real application, this would fetch from the Axum backend or Prisma.
  return NextResponse.json({
    referralClicks: id === "empty" ? 0 : 1245,
    ticketSales: id === "empty" ? 0 : 86,
    conversionRate: id === "empty" ? 0 : 6.9,
    totalCommissionEarned: id === "empty" ? 0 : 430.50,
  });
}
