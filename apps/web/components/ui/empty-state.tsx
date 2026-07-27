import React from "react";
import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";

interface EmptyStateAction {
  label: string;
  onClick?: () => void;
  href?: string;
}

interface EmptyStateProps {
  icon?: React.ReactNode;
  title: string;
  description: string;
  action?: EmptyStateAction;
  illustrationSrc?: string;
}

const DEFAULT_ILLUSTRATION = "/icons/404-illustration.svg";

export function EmptyState({
  icon,
  title,
  description,
  action,
  illustrationSrc = DEFAULT_ILLUSTRATION,
}: EmptyStateProps) {
  return (
    <div
      data-testid="empty-state"
      className="flex flex-col items-center justify-center gap-6 py-20 px-6 text-center"
    >
      {/* Illustration */}
      <div className="w-40 h-40 flex items-center justify-center">
        {icon ? (
          icon
        ) : (
          <Image
            src={illustrationSrc}
            width={160}
            height={160}
            alt="Empty state illustration"
            className="w-full h-full object-contain opacity-80"
          />
        )}
      </div>

      {/* Text content */}
      <div className="flex flex-col items-center gap-2 max-w-sm">
        <h3 className="text-xl font-semibold text-ink-deep">{title}</h3>
        <p className="text-sm text-muted-text leading-relaxed">{description}</p>
      </div>

      {/* Optional CTA */}
      {action && (
        action.href ? (
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
        )
      )}
    </div>
  );
}