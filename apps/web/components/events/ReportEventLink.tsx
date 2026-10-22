"use client";

import React, { useState } from "react";
import { useTranslations } from "next-intl";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { useFocusTrap } from "@/hooks/useFocusTrap";

interface ReportEventLinkProps {
  eventId: string | number;
  className?: string;
}

const REPORT_REASONS = ["Spam", "Scam", "Inappropriate", "Cancelled"] as const;

export function ReportEventLink({ eventId, className = "" }: ReportEventLinkProps) {
  const t = useTranslations("eventDetail");
  const [isOpen, setIsOpen] = useState(false);
  const [reason, setReason] = useState<string>("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleClose = () => {
    setIsOpen(false);
    setReason("");
    setError(null);
  };

  const modalRef = useFocusTrap<HTMLDivElement>(isOpen, handleClose);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!reason) {
      setError("Please select a reason.");
      return;
    }

    setIsSubmitting(true);
    try {
      const response = await fetch(`/api/v1/events/${eventId}/flag`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ flagged: true, reason }),
      });

      if (!response.ok) {
        const data = await response.json().catch(() => ({}));
        throw new Error(data.message || data.error || `Failed to report event (${response.status})`);
      }

      toast.success("Thanks, this event has been reported.");
      handleClose();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to report event.";
      setError(msg);
      toast.error(msg);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <>
      <button
        type="button"
        onClick={() => setIsOpen(true)}
        className={`text-xs text-gray-500 hover:text-gray-700 underline underline-offset-2 transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent ${className}`}
        aria-label={`${t("reportEvent")} (${eventId})`}
        aria-haspopup="dialog"
      >
        {t("reportEvent")}
      </button>

      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6">
          <div
            onClick={handleClose}
            className="fixed inset-0 bg-black/60 backdrop-blur-sm"
          />

          <div
            ref={modalRef}
            role="dialog"
            aria-modal="true"
            aria-labelledby="report-event-title"
            className="relative w-full max-w-md overflow-hidden rounded-2xl bg-white p-6 shadow-2xl border border-border-warm z-10"
          >
            <div className="flex items-center justify-between border-b border-border-warm pb-4">
              <h2 id="report-event-title" className="text-xl font-bold text-ink-soft">
                {t("reportEvent")}
              </h2>
              <button
                type="button"
                onClick={handleClose}
                className="rounded-full p-1.5 text-muted-text hover:bg-surface transition-colors"
                aria-label="Close modal"
              >
                ✕
              </button>
            </div>

            <form onSubmit={handleSubmit} className="mt-4 space-y-4">
              <fieldset className="space-y-2">
                <legend className="text-sm font-semibold text-ink-soft mb-2">
                  Why are you reporting this event?
                </legend>
                {REPORT_REASONS.map((option) => (
                  <label
                    key={option}
                    className="flex items-center gap-3 rounded-xl border border-border-warm px-4 py-2.5 text-sm text-ink-soft cursor-pointer hover:bg-surface"
                  >
                    <input
                      type="radio"
                      name="report-reason"
                      value={option}
                      checked={reason === option}
                      onChange={(e) => setReason(e.target.value)}
                    />
                    {option}
                  </label>
                ))}
              </fieldset>

              {error && (
                <div role="alert" className="rounded-xl bg-error/10 border border-error/30 p-3 text-xs text-error">
                  {error}
                </div>
              )}

              <div className="flex items-center gap-3 pt-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleClose}
                  disabled={isSubmitting}
                  className="flex-1"
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  disabled={isSubmitting || !reason}
                  className="flex-1 bg-accent text-ink-soft hover:bg-accent-hover font-semibold"
                >
                  {isSubmitting ? "Submitting..." : "Submit Report"}
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </>
  );
}
