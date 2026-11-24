"use client";

import React from "react";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import { useTranslations } from "next-intl";

interface ShareButtonProps {
  title: string;
  text?: string;
  url?: string;
  className?: string;
}

export default function ShareButton({
  title,
  text,
  url: customUrl,
  className = "px-4 py-2",
}: ShareButtonProps) {
  const t = useTranslations("eventDetail");

  const handleShare = async () => {
    const url = customUrl || (typeof window !== "undefined" ? window.location.href : "");

    try {
      // 1. Prioritize Web Share API when available
      if (typeof navigator !== "undefined" && typeof navigator.share === "function") {
        const shareData: ShareData = {
          title,
          ...(text ? { text } : {}),
          ...(url ? { url } : {}),
        };

        if (!navigator.canShare || navigator.canShare(shareData)) {
          await navigator.share(shareData);
          return;
        }
      }

      // 2. Fallback: Clipboard API
      if (typeof navigator !== "undefined" && navigator.clipboard?.writeText && url) {
        await navigator.clipboard.writeText(url);
        toast.success("Link copied to clipboard");
        return;
      }

      // 3. Fallback: Legacy execCommand
      if (url) {
        const input = document.createElement("input");
        input.value = url;
        input.setAttribute("readonly", "");
        input.style.position = "absolute";
        input.style.left = "-9999px";
        document.body.appendChild(input);
        input.select();
        document.execCommand("copy");
        document.body.removeChild(input);
        toast.success("Link copied to clipboard");
      }
    } catch (err) {
      // User cancelled share sheet — not an application error
      if (
        (err instanceof DOMException || err instanceof Error) &&
        (err.name === "AbortError" || err.name === "NotAllowedError")
      ) {
        return;
      }
      toast.error("Unable to share link");
    }
  };

  return (
    <Button variant="ghost" onClick={handleShare} className={className}>
      {t("share")}
    </Button>
  );
}
