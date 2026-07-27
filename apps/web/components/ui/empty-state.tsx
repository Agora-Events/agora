"use client";

import React from "react";
import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";

interface EmptyStateAction {
  label: string;
  onClick?: () => void;
  href?: string;
}

export interface EmptyStateProps {
  title: string;
  description?: string;
  message?: string;
  icon?: React.ReactNode;
  illustrationSrc?: string;
  action?: EmptyStateAction;
  ctaLabel?: string;
  ctaLink?: string;
}

const DEFAULT_ILLUSTRATION = "/icons/404-illustration.svg";

export function EmptyState({
  title,
  description,
  message,
  icon,
  illustrationSrc = DEFAULT_ILLUSTRATION,
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
            alt="Empty state illustration"
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

      {action &&
        (action.href ? (
          <Link
            href={action.href}
            className="inline-flex items-center gap-2 px-6 py-3 rounded-full bg-black text-white font-semibold text-sm shadow-[-4px_4px_0px_0px_rgba(0,0,0,0.4)] hover:-translate-x-[2px] hover:translate-y-[2px] active:-translate-x-[4px] active:translate-y-[4px] transition-transform"
          >
            {action.label}
          </Link>
        ) : (
          <Button variant="primary" onClick={action.onClick}>
            {action.label}
          </Button>
        ))}

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