import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom";
import { PriceTag } from "@/components/ui/PriceTag";
import * as usdcRateModule from "@/utils/usdc-rate";

// ─── Helpers ──────────────────────────────────────────────────────────────────

function mockRate(
  rate: number,
  isFallback = false,
  currency: usdcRateModule.FiatCurrency = "usd",
) {
  vi.spyOn(usdcRateModule, "fetchUsdcRate").mockResolvedValue({
    rate,
    isFallback,
    currency,
  });
}

// ─── Tests ────────────────────────────────────────────────────────────────────

describe("PriceTag", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  // ── USDC display ────────────────────────────────────────────────────────────

  it("displays the USDC amount formatted to 2 decimal places", async () => {
    mockRate(1.0);
    render(<PriceTag amountUsdc={25} />);
    expect(screen.getByText(/25\.00/)).toBeInTheDocument();
    expect(screen.getByText(/USDC/)).toBeInTheDocument();
  });

  it("formats zero USDC correctly", async () => {
    mockRate(1.0);
    render(<PriceTag amountUsdc={0} />);
    expect(screen.getByText(/0\.00/)).toBeInTheDocument();
  });

  it("formats fractional USDC amounts correctly", async () => {
    mockRate(1.0);
    render(<PriceTag amountUsdc={9.5} />);
    expect(screen.getByText(/9\.50/)).toBeInTheDocument();
  });

  // ── Fiat conversion display ─────────────────────────────────────────────────

  it("displays converted USD amount after rate loads", async () => {
    mockRate(1.001, false, "usd");
    render(<PriceTag amountUsdc={25} currency="usd" />);
    await waitFor(() =>
      expect(screen.getByText(/\$25\.03/)).toBeInTheDocument(),
    );
    // The fiat span contains "USD" (not "USDC"), check via the aria-label on the root
    expect(screen.getByLabelText(/approximately \$25\.03 USD/i)).toBeInTheDocument();
  });

  it("displays converted EUR amount after rate loads", async () => {
    mockRate(0.924, false, "eur");
    render(<PriceTag amountUsdc={100} currency="eur" />);
    await waitFor(() =>
      expect(screen.getByText(/€92\.40/)).toBeInTheDocument(),
    );
    expect(screen.getByText(/EUR/)).toBeInTheDocument();
  });

  it("rounds rounding-error-prone values safely", async () => {
    // 10 * 0.333 = 3.33, not 3.330000000000001
    mockRate(0.333, false, "usd");
    render(<PriceTag amountUsdc={10} currency="usd" />);
    await waitFor(() =>
      expect(screen.getByText(/\$3\.33/)).toBeInTheDocument(),
    );
  });

  // ── Loading skeleton ─────────────────────────────────────────────────────────

  it("shows a loading state before the rate resolves", () => {
    // Never-resolving promise simulates a slow network
    vi.spyOn(usdcRateModule, "fetchUsdcRate").mockReturnValue(
      new Promise(() => {}),
    );
    render(<PriceTag amountUsdc={25} />);
    // The skeleton is inside an aria-hidden span, so we need hidden:true
    expect(
      screen.getByRole("status", { hidden: true }),
    ).toBeInTheDocument();
  });

  // ── Fallback notice ─────────────────────────────────────────────────────────

  it("shows (est.) indicator when rate is a fallback", async () => {
    mockRate(1.0, true, "usd");
    render(<PriceTag amountUsdc={25} currency="usd" />);
    await waitFor(() =>
      expect(screen.getByText(/\(est\.\)/i)).toBeInTheDocument(),
    );
  });

  it("does not show (est.) indicator when rate is live", async () => {
    mockRate(1.001, false, "usd");
    render(<PriceTag amountUsdc={25} currency="usd" />);
    await waitFor(() =>
      expect(screen.queryByText(/\(est\.\)/i)).not.toBeInTheDocument(),
    );
  });

  // ── Accessible label ─────────────────────────────────────────────────────────

  it("includes an accessible aria-label with the full price information", async () => {
    mockRate(1.001, false, "usd");
    render(<PriceTag amountUsdc={25} currency="usd" />);
    await waitFor(() => {
      const el = screen.getByLabelText(/25\.00 USDC/i);
      expect(el).toBeInTheDocument();
    });
  });

  // ── className forwarding ─────────────────────────────────────────────────────

  it("applies custom className to the root element", async () => {
    mockRate(1.0);
    const { container } = render(
      <PriceTag amountUsdc={10} className="my-custom-class" />,
    );
    expect(container.firstChild).toHaveClass("my-custom-class");
  });
});
