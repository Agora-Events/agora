"use client";

import { useState } from "react";

export interface ExportAttendeesButtonProps {
  eventId: string;
  className?: string;
  variant?: "primary" | "secondary" | "outline";
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}

export function ExportAttendeesButton({
  eventId,
  className = "",
  variant = "primary",
  onSuccess,
  onError,
}: ExportAttendeesButtonProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const handleDownload = async () => {
    if (isLoading) return;

    setIsLoading(true);
    setErrorMessage(null);

    try {
      const response = await fetch(`/api/v1/events/${eventId}/export-attendees`, {
        method: "GET",
        credentials: "include",
      });

      if (!response.ok) {
        const errorText = await response.text().catch(() => "");
        throw new Error(errorText || `Failed to export attendees (${response.status})`);
      }

      const disposition = response.headers.get("Content-Disposition");
      let filename = `attendees-${eventId}.csv`;
      if (disposition) {
        const match = disposition.match(/filename=["']?([^"';]+)["']?/i);
        if (match && match[1]) {
          filename = match[1].trim();
        }
      }

      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = filename;
      document.body.appendChild(anchor);
      anchor.click();
      document.body.removeChild(anchor);
      window.URL.revokeObjectURL(url);

      onSuccess?.();
    } catch (err) {
      const error = err instanceof Error ? err : new Error("Failed to export attendees");
      setErrorMessage(error.message);
      onError?.(error);
    } finally {
      setIsLoading(false);
    }
  };

  const variantStyles = {
    primary:
      "bg-accent text-black border-2 border-black shadow-[-3px_3px_0_rgba(0,0,0,1)] hover:shadow-none hover:translate-x-[2px] hover:translate-y-[2px]",
    secondary:
      "bg-black text-white border-2 border-black shadow-[-3px_3px_0_rgba(0,0,0,1)] hover:shadow-none hover:translate-x-[2px] hover:translate-y-[2px]",
    outline:
      "bg-white text-ink-deep border-2 border-black shadow-[-3px_3px_0_rgba(0,0,0,1)] hover:shadow-none hover:translate-x-[2px] hover:translate-y-[2px]",
  }[variant];

  return (
    <div className="inline-flex flex-col items-start gap-1">
      <button
        type="button"
        onClick={handleDownload}
        disabled={isLoading}
        aria-label="Export Attendees (CSV)"
        aria-busy={isLoading}
        className={`inline-flex items-center justify-center gap-2 px-4 py-2 text-sm font-semibold rounded-xl transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed ${variantStyles} ${className}`}
      >
        {isLoading ? (
          <>
            <SpinnerIcon className="h-4 w-4 animate-spin text-current" />
            <span>Exporting...</span>
          </>
        ) : (
          <>
            <DownloadIcon className="h-4 w-4 text-current" />
            <span>Export Attendees (CSV)</span>
          </>
        )}
      </button>

      {errorMessage && (
        <span role="alert" className="text-xs text-red-600 font-medium">
          {errorMessage}
        </span>
      )}
    </div>
  );
}

function DownloadIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden
    >
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="7 10 12 15 17 10" />
      <line x1="12" y1="15" x2="12" y2="3" />
    </svg>
  );
}

function SpinnerIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden
    >
      <path d="M21 12a9 9 0 1 1-6.219-8.56" />
    </svg>
  );
}
