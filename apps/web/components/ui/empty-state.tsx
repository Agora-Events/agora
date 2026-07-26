"use client";

import React from "react";
import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";

interface EmptyStateAction {
  label: string;
  onClick: () => void;
}

export interface EmptyStateProps {
  /** Heading text */
  title: string;
  /**
   * Supporting body copy.
   * Prefer `description`; `message` is accepted as an alias for callers
   * that used the legacy PascalCase EmptyState API.
   */
  description?: string;
  message?: string;
  /** Custom icon node (preferred when providing an action button) */
  icon?: React.ReactNode;
  /** Override the default illustration when no `icon` is provided */
  illustrationSrc?: string;
  /** Optional click-handler CTA */
  action?: EmptyStateAction;
  /** Optional link CTA label (legacy API) */
  ctaLabel?: string;
  /** Optional link CTA href (legacy API) */
  ctaLink?: string;
}

/**
 * EmptyState — single shared empty-list placeholder.
 *
 * Supports both the icon + action pattern and the illustration + link CTA
 * pattern used across Discover, chat, and popular events.
 */
export function EmptyState({
  title,
  description,
  message,
  icon,
  illustrationSrc = "/icons/404-illustration.svg",
  action,
  ctaLabel,
  ctaLink,
}: EmptyStateProps) {
  const body = description ?? message ?? "";

  return (
    <div
      data-testid="empty-state"
      className="flex flex-col items-center justify-center gap-6 py-16 px-6 text-center"
    >
      {icon ? (
        <div className="w-20 h-20 rounded-full bg-surface flex items-center justify-center">
          {icon}
        </div>
      ) : (
        <div className="w-40 h-40 flex items-center justify-center">
          <Image
            src={illustrationSrc}
            width={160}
            height={160}
            alt=""
            className="w-full h-full object-contain opacity-80"
          />
        </div>
      )}

      <div className="flex flex-col items-center gap-2 max-w-sm">
        <h3 className="text-lg font-semibold text-ink-deep">{title}</h3>
        {body ? (
          <p className="text-sm text-ink-deep/50 leading-relaxed">{body}</p>
        ) : null}
      </div>

      {action ? (
        <Button variant="primary" onClick={action.onClick}>
          {action.label}
        </Button>
      ) : null}

      {!action && ctaLabel && ctaLink ? (
        <Link
          href={ctaLink}
          className="inline-flex items-center gap-2 px-6 py-3 rounded-full bg-black text-white font-semibold text-sm shadow-[-4px_4px_0px_0px_rgba(0,0,0,0.4)] hover:-translate-x-[2px] hover:translate-y-[2px] active:-translate-x-[4px] active:translate-y-[4px] transition-transform"
        >
          {ctaLabel}
        </Link>
      ) : null}
    </div>
  );
}
