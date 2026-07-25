import React from "react";
import Image from "next/image";
import Link from "next/link";

import { Button } from "@/components/ui/button";

interface EmptyStateAction {
  label: string;
  href?: string;
  onClick?: () => void;
}

interface EmptyStateProps {
  /** Heading text */
  title: string;

  /** Supporting message */
  description: string;

  /** Optional icon */
  icon?: React.ReactNode;

  /** Optional illustration image */
  illustrationSrc?: string;

  /** Optional action button or link */
  action?: EmptyStateAction;
}

export function EmptyState({ title, description, icon, illustrationSrc, action }: EmptyStateProps) {
  return (
    <div
      data-testid="empty-state"
      className="flex flex-col items-center justify-center gap-6 py-16 px-6 text-center"
    >
      {illustrationSrc ? (
        <div className="w-40 h-40 flex items-center justify-center">
          <Image
            src={illustrationSrc}
            width={160}
            height={160}
            alt="No events illustration"
            className="w-full h-full object-contain opacity-80"
          />
        </div>
      ) : (
        icon && (
          <div className="w-20 h-20 rounded-full bg-surface flex items-center justify-center">
            {icon}
          </div>
        )
      )}

      <div className="flex flex-col items-center gap-2 max-w-sm">
        <h3 className="text-xl font-semibold text-ink-deep">
          {title}
        </h3>
        <p className="text-sm text-muted-text leading-relaxed">
          {description}
        </p>
      </div>

      {action?.href ? (
        <Link
          href={action.href}
          className="inline-flex items-center gap-2 px-6 py-3 rounded-full bg-black text-white font-semibold text-sm shadow-[-4px_4px_0px_0px_rgba(0,0,0,0.4)] hover:-translate-x-[2px] hover:translate-y-[2px] active:-translate-x-[4px] active:translate-y-[4px] transition-transform"
        >
          {action.label}
        </Link>
      ) : action?.onClick ? (
        <Button variant="primary" onClick={action.onClick}>
          {action.label}
        </Button>
      ) : null}
    </div>
  );
}
