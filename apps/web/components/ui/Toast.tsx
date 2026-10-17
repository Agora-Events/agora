"use client";

/**
 * Toast — Issue #1542
 *
 * Animated toast notification for the transaction lifecycle.
 *
 * - Slides in from the bottom-right corner (translateY(100%) → 0).
 * - Slides out smoothly on dismiss (fade + translate).
 * - Shows a subtle spinning indicator while `status === "pending"`.
 * - Honors `prefers-reduced-motion` by disabling the slide animations.
 */

import { useCallback, useEffect, useRef, useState } from "react";

export type ToastStatus = "pending" | "success" | "error" | "info";

export interface ToastProps {
  /** Main message shown inside the toast. */
  message: string;
  /** Optional custom title to display, overriding default status label. */
  title?: string;
  /** Lifecycle status of the transaction this toast represents. */
  status?: ToastStatus;
  /**
   * Auto-dismiss delay in ms once the toast is no longer pending.
   * `null` (or a pending status) keeps the toast visible until dismissed.
   */
  duration?: number | null;
  /** Called after the exit animation completes. */
  onDismiss?: () => void;
  /** Optional additional CSS class names. */
  className?: string;
}

const EXIT_MS = 300;

const STATUS_STYLES: Record<ToastStatus, { border: string; label: string; icon: string }> = {
  pending: { border: "border-blue-400", label: "Pending", icon: "◌" },
  success: { border: "border-green-500", label: "Success", icon: "✓" },
  error: { border: "border-red-500", label: "Failed", icon: "✕" },
  info: { border: "border-zinc-400", label: "Info", icon: "ℹ" },
};

const TOAST_STYLES = `
@keyframes agora-toast-in {
  from { opacity: 0; transform: translateY(100%); }
  to   { opacity: 1; transform: translateY(0); }
}
@keyframes agora-toast-out {
  from { opacity: 1; transform: translateY(0); }
  to   { opacity: 0; transform: translateY(24px); }
}
@keyframes agora-toast-spin {
  to { transform: rotate(360deg); }
}
.agora-toast {
  animation: agora-toast-in 300ms cubic-bezier(0.21, 1.02, 0.73, 1) both;
}
.agora-toast[data-exiting="true"] {
  animation: agora-toast-out 300ms ease-in both;
}
.agora-toast-spinner {
  display: inline-block;
  width: 0.875rem;
  height: 0.875rem;
  border: 2px solid currentColor;
  border-right-color: transparent;
  border-radius: 9999px;
  animation: agora-toast-spin 0.7s linear infinite;
}
@media (prefers-reduced-motion: reduce) {
  .agora-toast,
  .agora-toast[data-exiting="true"] {
    animation: none;
  }
  .agora-toast-spinner {
    animation: none;
    border-right-color: currentColor;
    opacity: 0.6;
  }
}
`;

export function Toast({
  message,
  title,
  status = "info",
  duration = null,
  onDismiss,
  className = "",
}: ToastProps) {
  const [exiting, setExiting] = useState(false);
  const dismissRef = useRef(onDismiss);
  dismissRef.current = onDismiss;

  const startExit = useCallback(() => setExiting(true), []);

  // Run the exit animation, then notify the parent to unmount us.
  useEffect(() => {
    if (!exiting) return;
    const timer = setTimeout(() => dismissRef.current?.(), EXIT_MS);
    return () => clearTimeout(timer);
  }, [exiting]);

  // Auto-dismiss after `duration`, but never while the tx is still pending.
  useEffect(() => {
    if (status === "pending" || duration == null || exiting) return;
    const timer = setTimeout(startExit, duration);
    return () => clearTimeout(timer);
  }, [status, duration, exiting, startExit]);

  const { border, label, icon } = STATUS_STYLES[status];

  return (
    <>
      <style>{TOAST_STYLES}</style>
      <div
        role="status"
        aria-live="polite"
        data-status={status}
        data-exiting={exiting}
        className={`agora-toast pointer-events-auto flex w-full max-w-sm items-start gap-3 rounded-lg border-l-4 ${border} bg-zinc-900 p-4 text-zinc-100 shadow-lg ${className}`}
      >
        <span aria-hidden="true" className="mt-0.5 text-sm">
          {status === "pending" ? <span className="agora-toast-spinner" /> : icon}
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-sm font-semibold">{title || label}</p>
          <p className="mt-0.5 text-sm break-words text-zinc-300">{message}</p>
        </div>
        <button
          type="button"
          aria-label="Dismiss notification"
          onClick={startExit}
          className="rounded p-1 text-zinc-400 transition-colors hover:text-zinc-100"
        >
          ✕
        </button>
      </div>
    </>
  );
}

export interface ErrorToastProps extends Omit<ToastProps, "status" | "message"> {
  /** Main error message. Optional if error object or string is provided. */
  message?: string;
  /** Optional Error instance or error message string. */
  error?: Error | string;
}

/**
 * Standardized ErrorToast wrapper component for displaying error notifications.
 *
 * Refactored as part of Issue #1581.
 */
export function ErrorToast({
  message,
  error,
  title = "Error",
  duration = 4000,
  onDismiss,
  className = "",
  ...rest
}: ErrorToastProps) {
  const displayMessage =
    message ||
    (typeof error === "string" ? error : error?.message) ||
    "An unexpected error occurred.";

  return (
    <Toast
      message={displayMessage}
      title={title}
      status="error"
      duration={duration}
      onDismiss={onDismiss}
      className={className}
      {...rest}
    />
  );
}

export default Toast;
