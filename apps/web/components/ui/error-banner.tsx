"use client";

/**
 * ErrorBanner — accessible error alert with a retry button.
 * Displays when an API request fails, allowing users to re-execute the fetch.
 */
interface ErrorBannerProps {
  /** Primary error message to display */
  message: string;
  /** Optional detailed description of the error */
  description?: string;
  /** Callback fired when the "Retry" button is clicked */
  onRetry: () => void;
}

export function ErrorBanner({ message, description, onRetry }: ErrorBannerProps) {
  return (
    <div
      role="alert"
      className="w-full rounded-xl border border-red-300 bg-red-50 p-4 flex items-start gap-3"
    >
      {/* Error icon */}
      <svg
        className="w-5 h-5 text-red-500 mt-0.5 shrink-0"
        fill="none"
        viewBox="0 0 24 24"
        strokeWidth={2}
        stroke="currentColor"
        aria-hidden="true"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
        />
      </svg>

      <div className="flex-1 min-w-0">
        <p className="text-sm font-semibold text-red-800">{message}</p>
        {description && (
          <p className="text-sm text-red-600 mt-1">{description}</p>
        )}
      </div>

      <button
        type="button"
        onClick={onRetry}
        className="shrink-0 rounded-lg bg-red-600 px-4 py-2 text-xs font-semibold text-white shadow-sm hover:bg-red-700 focus-visible:outline-2 focus-visible:outline-red-500 transition-colors"
      >
        Retry
      </button>
    </div>
  );
}