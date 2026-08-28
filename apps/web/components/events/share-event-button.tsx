"use client";

import { useState } from "react";
import { toast } from "sonner";
import { Share2 } from "@/components/ui/icons";

interface ShareEventButtonProps {
  title: string;
}

/**
 * Share control for the event details page. Uses the Web Share API when
 * available (`navigator.share`), otherwise falls back to copying the page
 * URL to the clipboard and showing a success toast via the globally mounted
 * sonner Toaster.
 *
 * A user-cancelled share sheet (AbortError) is silently ignored.
 */
export function ShareEventButton({ title }: ShareEventButtonProps) {
  const [pending, setPending] = useState(false);

  const handleShare = async () => {
    const url = window.location.href;

    if (typeof navigator.share === "function") {
      setPending(true);
      try {
        await navigator.share({
          title,
          text: `Check out this event: ${title}`,
          url,
        });
      } catch (error) {
        // User dismissed the share sheet — not an error worth surfacing.
        // Note: DOMException does not satisfy `instanceof Error`, so check
        // the name alone.
        const errName = (error as { name?: string } | null)?.name;
        if (errName === "AbortError") {
          return;
        }
        // Some browsers throw for other reasons (e.g. not allowed);
        // fall back to the clipboard path below.
        await copyToClipboard(url);
        toast.success("Link copied");
      } finally {
        setPending(false);
      }
      return;
    }

    await copyToClipboard(url);
    toast.success("Link copied");
  };

  const copyToClipboard = async (url: string) => {
    try {
      await navigator.clipboard.writeText(url);
    } catch {
      // Clipboard API unavailable (e.g. insecure context) — last resort.
      const input = document.createElement("textarea");
      input.value = url;
      document.body.appendChild(input);
      input.select();
      document.execCommand("copy");
      document.body.removeChild(input);
    }
  };

  return (
    <button
      type="button"
      onClick={handleShare}
      disabled={pending}
      aria-label="Share this event"
      className="inline-flex shrink-0 items-center gap-2 rounded-full border-2 border-black bg-white px-4 py-2 text-sm font-semibold text-black shadow-[-3px_3px_0px_0px_rgba(0,0,0,1)] transition-all hover:-translate-x-[1px] hover:translate-y-[1px] hover:shadow-[-1px_1px_0px_0px_rgba(0,0,0,1)] active:translate-y-[2px] active:shadow-none focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-[#FDDA23]/60"
    >
      <Share2 size={18} className="text-black" />
      Share
    </button>
  );
}
