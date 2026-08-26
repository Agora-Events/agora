"use client";

import { useState, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { dataEvents } from "@/components/events/mockups";
import { ReferralSharingModal } from "./referral-sharing-modal";

interface ReferralLinkGeneratorProps {
  /** Optional pre-filled affiliate code from auth context */
  defaultAffiliateCode?: string;
}

/**
 * ReferralLinkGenerator — allows affiliates to generate a unique referral
 * link for any event using their affiliate code. Displays the generated URL
 * with a one-click copy-to-clipboard action.
 */
export function ReferralLinkGenerator({
  defaultAffiliateCode = "",
}: ReferralLinkGeneratorProps) {
  const [affiliateCode, setAffiliateCode] = useState(defaultAffiliateCode);
  const [selectedEventId, setSelectedEventId] = useState<number | "">("");
  const [generatedUrl, setGeneratedUrl] = useState("");
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState("");
  const [shareModalOpen, setShareModalOpen] = useState(false);

  /** Build the base URL for the current environment */
  const baseUrl =
    typeof window !== "undefined"
      ? `${window.location.protocol}//${window.location.host}`
      : "https://agora.app";

  /** Generate the referral link */
  const handleGenerate = useCallback(() => {
    setError("");
    setCopied(false);

    const trimmedCode = affiliateCode.trim();
    if (!trimmedCode) {
      setError("Please enter your affiliate code.");
      return;
    }
    if (!selectedEventId) {
      setError("Please select an event.");
      return;
    }

    const url = `${baseUrl}/events/${selectedEventId}?ref=${encodeURIComponent(trimmedCode)}`;
    setGeneratedUrl(url);
  }, [affiliateCode, selectedEventId, baseUrl]);

  /** Copy the generated URL to the clipboard */
  const handleCopy = useCallback(async () => {
    if (!generatedUrl) return;
    try {
      await navigator.clipboard.writeText(generatedUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback for older browsers
      const textArea = document.createElement("textarea");
      textArea.value = generatedUrl;
      textArea.style.position = "fixed";
      textArea.style.opacity = "0";
      document.body.appendChild(textArea);
      textArea.select();
      try {
        document.execCommand("copy");
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch {
        setError("Could not copy to clipboard. Please copy the link manually.");
      }
      document.body.removeChild(textArea);
    }
  }, [generatedUrl]);

  return (
    <div className="rounded-3xl border-2 border-black bg-white p-6 sm:p-8 shadow-[-6px_6px_0_rgba(0,0,0,1)]">
      {/* Header */}
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-ink-deep">Referral Link Generator</h2>
        <p className="mt-1 text-sm text-gray-500">
          Generate a unique referral link for any event and start earning commissions.
        </p>
      </div>

      {/* Affiliate Code Input */}
      <div className="mb-5">
        <label
          htmlFor="affiliate-code"
          className="mb-2 block text-sm font-medium text-black"
        >
          Your Affiliate Code
        </label>
        <input
          id="affiliate-code"
          type="text"
          value={affiliateCode}
          onChange={(e) => {
            setAffiliateCode(e.target.value);
            setError("");
          }}
          placeholder="e.g. demo123"
          className="w-full rounded-full border-2 border-black bg-white px-4 py-2 text-sm outline-none shadow-[4px_4px_0px_0px_#000] transition-shadow focus:shadow-[2px_2px_0px_0px_#000]"
        />
      </div>

      {/* Event Selector */}
      <div className="mb-6">
        <label
          htmlFor="event-select"
          className="mb-2 block text-sm font-medium text-black"
        >
          Select Event
        </label>
        <select
          id="event-select"
          value={selectedEventId}
          onChange={(e) => {
            setSelectedEventId(e.target.value ? Number(e.target.value) : "");
            setError("");
          }}
          className="w-full rounded-full border-2 border-black bg-white px-4 py-2 text-sm outline-none shadow-[4px_4px_0px_0px_#000] transition-shadow focus:shadow-[2px_2px_0px_0px_#000] appearance-none"
          style={{
            backgroundImage: `url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3e%3cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3e%3c/svg%3e")`,
            backgroundPosition: "right 12px center",
            backgroundRepeat: "no-repeat",
            backgroundSize: "20px",
          }}
        >
          <option value="">-- Choose an event --</option>
          {dataEvents.map((event) => (
            <option key={event.id} value={event.id}>
              {event.title} — {event.date} ({event.location})
            </option>
          ))}
        </select>
      </div>

      {/* Error Message */}
      {error && (
        <div
          role="alert"
          className="mb-4 rounded-xl border-2 border-red-300 bg-red-50 px-4 py-3 text-sm text-red-700"
        >
          {error}
        </div>
      )}

      {/* Generate Button */}
      <Button
        type="button"
        onClick={handleGenerate}
        backgroundColor="bg-violet-600"
        textColor="text-white"
        className="w-full"
      >
        Generate Referral Link
      </Button>

      {/* Generated URL Display */}
      {generatedUrl && (
        <div className="mt-6 rounded-2xl border-2 border-black/10 bg-surface p-4">
          <label className="mb-2 block text-xs font-bold uppercase tracking-[0.12em] text-gray-500">
            Your Referral Link
          </label>
          <div className="flex items-center gap-2">
            <input
              type="text"
              readOnly
              value={generatedUrl}
              className="flex-1 truncate rounded-lg border border-black/10 bg-white px-3 py-2 text-sm font-mono text-ink-deep outline-none"
              aria-label="Generated referral link"
            />
            <button
              type="button"
              onClick={handleCopy}
              className={`flex shrink-0 items-center gap-1.5 rounded-full border-2 px-4 py-2 text-sm font-medium transition-all ${
                copied
                  ? "border-green-500 bg-green-50 text-green-700"
                  : "border-black bg-white text-ink-deep hover:bg-gray-50"
              }`}
              aria-label={copied ? "Copied" : "Copy to clipboard"}
            >
              {copied ? (
                <>
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                  Copied!
                </>
              ) : (
                <>
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                  </svg>
                  Copy
                </>
              )}
            </button>
          </div>
          <p className="mt-2 text-xs text-gray-400">
            Share this link with your audience. You earn commissions on ticket
            sales made through this link.
          </p>

          {/* Share action */}
          <button
            type="button"
            onClick={() => setShareModalOpen(true)}
            className="mt-4 flex w-full items-center justify-center gap-2 rounded-full border-2 border-black bg-ink-deep px-4 py-2.5 text-sm font-semibold text-white transition-transform hover:-translate-y-0.5"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <circle cx="18" cy="5" r="3" />
              <circle cx="6" cy="12" r="3" />
              <circle cx="18" cy="19" r="3" />
              <path d="M8.59 13.51l6.83 3.98M15.41 6.51l-6.82 3.98" />
            </svg>
            Share Link
          </button>
        </div>
      )}

      {/* Referral sharing modal */}
      <ReferralSharingModal
        isOpen={shareModalOpen}
        onClose={() => setShareModalOpen(false)}
        referralUrl={generatedUrl}
        shareText="Join me at this event on Agora Events! Get your ticket here:"
      />
    </div>
  );
}