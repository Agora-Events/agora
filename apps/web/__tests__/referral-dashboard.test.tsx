import React from "react";
import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import ReferralDashboardPage from "../app/referrals/page";

vi.mock("@/components/layout/navbar", () => ({
  Navbar: () => <nav data-testid="mock-navbar">Navbar</nav>,
}));

vi.mock("@/components/layout/footer", () => ({
  Footer: () => <footer data-testid="mock-footer">Footer</footer>,
}));

vi.mock("@/components/affiliates/referral-link-generator", () => ({
  ReferralLinkGenerator: () => <div data-testid="mock-link-generator">Link Generator</div>,
}));

describe("ReferralDashboardPage Component", () => {
  it("renders page title and affiliate metrics", () => {
    render(<ReferralDashboardPage />);

    expect(screen.getByText("Earnings & Referrals")).toBeInTheDocument();
    expect(screen.getByText("Total Referral Clicks")).toBeInTheDocument();
    expect(screen.getByText("Total Commission Earned")).toBeInTheDocument();
    expect(screen.getByTestId("mock-link-generator")).toBeInTheDocument();
  });

  it("renders recent referral table activity", () => {
    render(<ReferralDashboardPage />);

    expect(screen.getByText("Recent Referral Activity")).toBeInTheDocument();
    expect(screen.getByText("Stellar Dev Summit 2026")).toBeInTheDocument();
  });
});
