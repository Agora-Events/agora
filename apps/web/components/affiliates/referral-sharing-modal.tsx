"use client";

import { useCallback, useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { useFocusTrap } from "@/hooks/useFocusTrap";
import { toast } from "sonner";

// ─── Types ────────────────────────────────────────────────────────────────────

interface ReferralSharingModalProps {
  /** Whether the modal is open */
  isOpen: boolean;
  /** Callback fired when the modal requests to close */
  onClose: () => void;
  /** The referral link to share (e.g. https://agora.app/events/1?ref=demo123) */
  referralUrl: string;
  /** Optional short message appended to platform share intents */
  shareText?: string;
}

// ─── Platform icons ─────────────────────────────────────────────────────────────

function WhatsAppIcon({ size = 20 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M12.04 2C6.58 2 2.13 6.45 2.13 11.91c0 1.75.46 3.45 1.32 4.95L2.05 22l5.25-1.38a9.87 9.87 0 0 0 4.74 1.21h.01c5.46 0 9.9-4.45 9.9-9.91 0-2.65-1.03-5.14-2.9-7.01A9.82 9.82 0 0 0 12.04 2zm0 18.15h-.01a8.2 8.2 0 0 1-4.19-1.15l-.3-.18-3.12.82.83-3.04-.2-.31a8.2 8.2 0 0 1-1.26-4.38c0-4.54 3.7-8.24 8.25-8.24 2.2 0 4.27.86 5.82 2.42a8.18 8.18 0 0 1 2.41 5.83c0 4.54-3.7 8.23-8.23 8.23zm4.52-6.16c-.25-.12-1.47-.72-1.69-.81-.23-.08-.39-.12-.56.13-.17.24-.64.8-.78.97-.14.16-.29.18-.54.06-.25-.12-1.05-.39-1.99-1.23-.74-.66-1.23-1.47-1.38-1.72-.14-.25-.02-.38.11-.51.11-.11.25-.29.37-.43.12-.14.17-.25.25-.41.08-.17.04-.31-.02-.43-.06-.12-.56-1.34-.76-1.84-.2-.48-.41-.42-.56-.43h-.48c-.17 0-.43.06-.66.31-.22.25-.86.85-.86 2.07 0 1.22.89 2.4 1.01 2.56.12.17 1.75 2.67 4.23 3.74.59.26 1.05.41 1.41.52.59.19 1.13.16 1.56.1.48-.07 1.47-.6 1.67-1.18.21-.58.21-1.07.15-1.18-.06-.1-.23-.16-.48-.29z" />
    </svg>
  );
}

function XIcon({ size = 20 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z" />
    </svg>
  );
}

function CopyIcon({ size = 20 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </svg>
  );
}

function LinkIcon({ size = 20 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
      <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
    </svg>
  );
}

// ─── Component ────────────────────────────────────────────────────────────────

/**
 * ReferralSharingModal — a reusable modal that lets affiliates share their
 * referral link via WhatsApp, X (Twitter), the native Web Share API (where
 * available), or a click-to-copy fallback for unsupported browsers.
 *
 * The modal is intentionally presentational: it receives the fully-formed
 * referral URL from its caller so it can be reused anywhere in the app.
 */
export function ReferralSharingModal({
  isOpen,
  onClose,
  referralUrl,
  shareText = "Check out this event — get your tickets here:",
}: ReferralSharingModalProps) {
  const modalRef = useFocusTrap<HTMLDivElement>(isOpen, onClose);
  const [copied, setCopied] = useState(false);
  const [showFallback, setShowFallback] = useState(false);

  const supportsNativeShare =
    typeof navigator !== "undefined" && typeof navigator.share === "function";

  // Reset transient state whenever the modal opens.
  useEffect(() => {
    if (isOpen) {
      setCopied(false);
      setShowFallback(false);
    }
  }, [isOpen]);

  /** Copy the referral URL to the clipboard (async API with textarea fallback). */
  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(referralUrl);
    } catch {
      // Fallback for older browsers / non-secure contexts.
      const textArea = document.createElement("textarea");
      textArea.value = referralUrl;
      textArea.style.position = "fixed";
      textArea.style.opacity = "0";
      document.body.appendChild(textArea);
      textArea.select();
      try {
        document.execCommand("copy");
      } finally {
        document.body.removeChild(textArea);
      }
    }
    setCopied(true);
    toast.success("Referral link copied to clipboard!");
    setTimeout(() => setCopied(false), 2000);
  }, [referralUrl]);

  /** Trigger the native Web Share API when the browser supports it. */
  const handleNativeShare = useCallback(async () => {
    if (!supportsNativeShare) {
      setShowFallback(true);
      return;
    }
    try {
      await navigator.share({
        title: "Agora Events",
        text: shareText,
        url: referralUrl,
      });
    } catch (error) {
      // AbortError means the user dismissed the share sheet — not a failure.
      if ((error as Error).name !== "AbortError") {
        // Fall back to the copy UI so the user can still share their link.
        setShowFallback(true);
      }
    }
  }, [shareText, referralUrl, supportsNativeShare]);

  if (!isOpen) return null;

  const intentText = encodeURIComponent(shareText);
  const intentUrl = encodeURIComponent(referralUrl);
  const whatsappHref = `https://wa.me/?text=${intentText}%20${intentUrl}`;
  const xHref = `https://twitter.com/intent/tweet?text=${intentText}&url=${intentUrl}`;

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 sm:p-6">
        {/* Backdrop */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="absolute inset-0 bg-black/60 backdrop-blur-md"
          aria-hidden="true"
        />

        {/* Modal */}
        <motion.div
          ref={modalRef}
          role="dialog"
          aria-modal="true"
          aria-labelledby="referral-share-title"
          initial={{ scale: 0.9, opacity: 0, y: 20 }}
          animate={{ scale: 1, opacity: 1, y: 0 }}
          exit={{ scale: 0.9, opacity: 0, y: 20 }}
          className="relative w-full max-w-[480px] rounded-[32px] border-2 border-black bg-white p-8 shadow-[-8px_8px_0_rgba(0,0,0,1)] sm:p-10"
        >
          {/* Close button */}
          <button
            type="button"
            onClick={onClose}
            className="absolute right-5 top-5 flex h-10 w-10 items-center justify-center rounded-full border border-black/10 bg-white/60 transition-colors hover:bg-gray-100"
            aria-label="Close sharing options"
          >
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>

          {/* Header */}
          <div className="mb-7">
            <div className="mb-2 flex items-center gap-2 text-violet-700">
              <LinkIcon size={18} />
              <span className="text-xs font-bold uppercase tracking-[0.18em]">
                Share
              </span>
            </div>
            <h2
              id="referral-share-title"
              className="text-[28px] font-bold leading-tight text-ink-deep sm:text-[32px]"
            >
              Share your referral link
            </h2>
            <p className="mt-2 text-sm text-gray-500">
              Spread the word and earn commission on every ticket sold through
              your link.
            </p>
          </div>

          {/* Platform buttons */}
          <div className="flex flex-col gap-3">
            {/* Native Web Share (primary on supporting devices) */}
            {supportsNativeShare && (
              <button
                type="button"
                onClick={handleNativeShare}
                className="flex w-full items-center justify-between rounded-2xl border-2 border-black bg-ink-deep px-5 py-4 text-left font-semibold text-white transition-transform hover:-translate-y-0.5"
              >
                <span className="flex items-center gap-3">
                  <svg
                    width="20"
                    height="20"
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
                  Share via…
                </span>
                <span aria-hidden="true">↗</span>
              </button>
            )}

            {/* WhatsApp */}
            <a
              href={whatsappHref}
              target="_blank"
              rel="noopener noreferrer"
              className="flex w-full items-center justify-between rounded-2xl border-2 border-black bg-white px-5 py-4 font-semibold text-black transition-transform hover:-translate-y-0.5"
            >
              <span className="flex items-center gap-3">
                <span className="text-green-600">
                  <WhatsAppIcon size={22} />
                </span>
                WhatsApp
              </span>
              <span aria-hidden="true">↗</span>
            </a>

            {/* X (Twitter) */}
            <a
              href={xHref}
              target="_blank"
              rel="noopener noreferrer"
              className="flex w-full items-center justify-between rounded-2xl border-2 border-black bg-white px-5 py-4 font-semibold text-black transition-transform hover:-translate-y-0.5"
            >
              <span className="flex items-center gap-3">
                <span className="text-black">
                  <XIcon size={18} />
                </span>
                X (Twitter)
              </span>
              <span aria-hidden="true">↗</span>
            </a>

            {/* Fallback: copy link (always available) */}
            {!supportsNativeShare && (
              <button
                type="button"
                onClick={handleCopy}
                className="flex w-full items-center justify-between rounded-2xl border-2 border-black bg-white px-5 py-4 font-semibold text-black transition-transform hover:-translate-y-0.5"
              >
                <span className="flex items-center gap-3">
                  <span className="text-gray-600">
                    <CopyIcon size={20} />
                  </span>
                  {copied ? "Copied!" : "Copy Link"}
                </span>
                {copied && (
                  <span className="text-green-600" aria-live="polite">
                    ✓
                  </span>
                )}
              </button>
            )}
          </div>

          {/* Unsupported-browser fallback panel */}
          {!supportsNativeShare && (
            <div className="mt-5 rounded-2xl border border-black/10 bg-surface px-5 py-4">
              <p className="mb-3 text-xs font-bold uppercase tracking-[0.12em] text-gray-500">
                Or copy the link
              </p>
              <div className="flex items-center gap-2">
                <input
                  type="text"
                  readOnly
                  value={referralUrl}
                  aria-label="Referral link"
                  className="min-w-0 flex-1 truncate rounded-lg border border-black/10 bg-white px-3 py-2 font-mono text-xs text-ink-deep outline-none"
                />
                <button
                  type="button"
                  onClick={handleCopy}
                  className={`flex shrink-0 items-center gap-1.5 rounded-full border-2 px-4 py-2 text-sm font-medium transition-colors ${
                    copied
                      ? "border-green-500 bg-green-50 text-green-700"
                      : "border-black bg-white hover:bg-gray-50"
                  }`}
                  aria-label={copied ? "Link copied" : "Copy referral link"}
                >
                  {copied ? "Copied" : "Copy"}
                </button>
              </div>
            </div>
          )}

          {/* Native-share failure fallback */}
          {supportsNativeShare && showFallback && (
            <div className="mt-5 rounded-2xl border border-black/10 bg-surface px-5 py-4">
              <p className="mb-3 text-xs font-bold uppercase tracking-[0.12em] text-gray-500">
                Copy the link instead
              </p>
              <Button
                type="button"
                onClick={handleCopy}
                variant="secondary"
                className="w-full"
              >
                {copied ? "Copied!" : "Copy Link"}
              </Button>
            </div>
          )}
        </motion.div>
      </div>
    </AnimatePresence>
  );
}