"use client";

import React from "react";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";

interface CardOnrampProps {
  amountUsd: number;
  receivingAddress: string; // smart contract or platform receiving address
  onSuccess: (fundedWallet: string) => void;
  onCancel?: () => void;
  onError?: (err: Error) => void;
}

/**
 * Lightweight wrapper for a fiat-to-crypto onramp widget.
 *
 * NOTE: This component contains a simple UI and hooks where a real
 * MoonPay / Stripe onramp integration can be added. It intentionally
 * does not ship API keys. Replace the stubbed `openWidget` logic with
 * the official provider SDK according to their docs.
 */
export default function CardOnramp({ amountUsd, receivingAddress, onSuccess, onCancel, onError }: CardOnrampProps) {
  const openWidget = async () => {
    try {
      // Real integration point:
      // - Load provider SDK (MoonPay/Stripe) dynamically here
      // - Configure the widget with `amountUsd` and `receivingAddress`
      // - Open the widget and listen for success/cancel/error events

      // STUB: Simulate user completing a purchase and the onramp
      // delivering funds to a newly created wallet address.
      toast('Opening payment widget...');
      await new Promise((r) => setTimeout(r, 1200));

      // Simulated funded wallet address returned by the onramp provider
      const simulatedFundedWallet = "GMOONPAYFUNDEDWALLETEXAMPLE000000000";

      // Simulate success callback
      onSuccess(simulatedFundedWallet);
    } catch (err) {
      const error = err instanceof Error ? err : new Error("Onramp failed");
      onError?.(error);
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm text-black/60">
        Pay with a credit or debit card. The onramp provider will convert your
        fiat payment into USDC and deposit it into a Stellar wallet so Agora can
        complete the purchase. No card details are processed by Agora's backend.
      </p>

      <div className="flex gap-2">
        <Button variant="secondary" onClick={openWidget} className="flex-1">
          Pay ${amountUsd.toFixed(2)} with Card
        </Button>
        <Button
          variant="ghost"
          onClick={() => {
            onCancel?.();
          }}
        >
          Cancel
        </Button>
      </div>
    </div>
  );
}
