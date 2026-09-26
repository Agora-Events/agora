"use client";

import Image from "next/image";
import { LazyImage } from "@/components/ui/LazyImage";

export interface PoapCollectible {
  id: string;
  payment_id: string;
  event_id: string;
  event_title: string;
  event_image_url?: string | null;
  event_location?: string | null;
  event_date?: string | null;
  minted_at: string;
  token_id: string;
  badge_url?: string;
  is_soulbound?: boolean;
}

export interface PoapCardProps {
  poap: PoapCollectible;
  onClick?: () => void;
  className?: string;
}

function formatDate(iso?: string | null): string {
  if (!iso) return "Date N/A";
  try {
    return new Intl.DateTimeFormat("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
    }).format(new Date(iso));
  } catch {
    return iso;
  }
}

export function PoapCard({ poap, onClick, className = "" }: PoapCardProps) {
  const formattedDate = formatDate(poap.event_date || poap.minted_at);

  return (
    <article
      role={onClick ? "button" : undefined}
      tabIndex={onClick ? 0 : undefined}
      aria-label={`POAP NFT for ${poap.event_title}`}
      onClick={onClick}
      onKeyDown={onClick ? (e) => e.key === "Enter" && onClick() : undefined}
      className={[
        "group relative flex flex-col justify-between overflow-hidden rounded-2xl border border-border-warm bg-gradient-to-b from-white to-surface/40 p-5",
        "shadow-[-4px_4px_0_rgba(0,0,0,0.06)] transition-all duration-200",
        "hover:border-accent/60 hover:shadow-[-6px_6px_0_rgba(0,0,0,0.1)] hover:-translate-y-0.5",
        onClick ? "cursor-pointer" : "",
        className,
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {/* Top row: Soulbound Badge & Verified Icon */}
      <div className="flex items-center justify-between gap-2 mb-4">
        <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-bold tracking-wide uppercase bg-accent/20 text-ink-soft border border-accent/30">
          <Image
            src="/icons/ticket-star.svg"
            width={12}
            height={12}
            alt=""
            aria-hidden="true"
          />
          POAP NFT
        </span>
        <span
          className="inline-flex items-center gap-1 text-[11px] font-medium text-emerald-600 bg-emerald-50 px-2 py-0.5 rounded-full"
          title="Verified Attendance"
        >
          <Image
            src="/icons/check-circle.svg"
            width={12}
            height={12}
            alt=""
            aria-hidden="true"
          />
          Verified
        </span>
      </div>

      {/* Center: POAP Medallion & Event Title */}
      <div className="flex items-center gap-4 my-2">
        {/* Medallion Avatar */}
        <div className="relative flex-shrink-0 w-16 h-16 rounded-full p-1 bg-gradient-to-tr from-accent via-amber-200 to-yellow-400 shadow-sm group-hover:scale-105 transition-transform duration-200">
          <div className="w-full h-full rounded-full overflow-hidden bg-white flex items-center justify-center">
            {poap.badge_url || poap.event_image_url ? (
              <LazyImage
                src={poap.badge_url || poap.event_image_url!}
                alt={poap.event_title}
                width={64}
                height={64}
                className="object-cover w-full h-full"
              />
            ) : (
              <Image
                src="/icons/ticket-star.svg"
                width={28}
                height={28}
                alt=""
                className="opacity-70"
              />
            )}
          </div>
        </div>

        {/* Info */}
        <div className="flex-1 min-w-0">
          <h4 className="font-bold text-base text-ink-soft leading-snug line-clamp-1 group-hover:text-accent-hover transition-colors">
            {poap.event_title}
          </h4>
          <p className="text-xs text-muted-text mt-0.5 truncate">
            {poap.event_location || "Attended Event"}
          </p>
          <p className="text-[11px] text-muted-text/80 mt-1 font-mono">
            ID: #{poap.token_id.slice(0, 10)}
          </p>
        </div>
      </div>

      {/* Bottom row: Non-transferable indicator & Minted Date */}
      <div className="mt-4 pt-3 border-t border-border-warm/60 flex items-center justify-between text-xs text-muted-text">
        <span className="flex items-center gap-1 text-[11px]">
          <Image
            src="/icons/lock.svg"
            width={11}
            height={11}
            alt=""
            className="opacity-60"
          />
          Non-transferable
        </span>
        <span className="text-[11px] font-medium text-ink-soft/70">
          {formattedDate}
        </span>
      </div>
    </article>
  );
}

export default PoapCard;
