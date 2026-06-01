"use client";

import Link from "next/link";
import Image from "next/image";
import { NavLink } from "./nav-link";

// ─── Types ────────────────────────────────────────────────────────────────────

export interface NavItem {
  href: string;
  icon: string;
  text: string;
  /** Determines active highlight; evaluated against the current pathname */
  isActive: (pathname: string) => boolean;
}

export interface BaseNavProps {
  pathname: string;
  isAuthenticated: boolean;
  navItems: NavItem[];
  /** Slot rendered to the right of the nav links (e.g. CTA button) */
  ctaSlot: React.ReactNode;
  /** Optional slot rendered on the far right (e.g. notifications + avatar) */
  endSlot?: React.ReactNode;
}

// ─── Component ────────────────────────────────────────────────────────────────

/**
 * BaseNav — shared desktop navigation shell.
 *
 * Renders the Agora logo, a list of NavLink items, and two optional slots
 * (ctaSlot for the primary CTA button, endSlot for auth controls).
 * GuestNav and UserNav are thin wrappers that pre-fill these props.
 */
export function BaseNav({
  pathname,
  isAuthenticated,
  navItems,
  ctaSlot,
  endSlot,
}: BaseNavProps) {
  return (
    <div
      className={`flex items-center w-full ${
        isAuthenticated ? "justify-between" : "gap-[231px]"
      }`}
    >
      {/* ── Logo ── */}
      <div
        className={
          isAuthenticated ? "flex items-center gap-8 lg:gap-[137px]" : undefined
        }
      >
        <Link href="/" className="flex items-center z-50">
          <Image
            src="/logo/agora logo.svg"
            alt="Agora Logo"
            width={100}
            height={30}
            className="h-auto w-auto"
          />
        </Link>

        {/* ── Nav links + CTA ── */}
        <div
          className={`hidden lg:flex items-center ${
            isAuthenticated ? "gap-[53px]" : "flex-1 gap-[170px]"
          }`}
        >
          <div
            className={`flex items-center ${
              isAuthenticated ? "gap-6" : "gap-[25px]"
            }`}
          >
            {navItems.map((item) => (
              <NavLink
                key={item.href + item.text}
                href={item.href}
                icon={item.icon}
                text={item.text}
                isActive={item.isActive(pathname)}
              />
            ))}
          </div>

          {ctaSlot}
        </div>
      </div>

      {/* ── End slot (notifications, avatar, etc.) ── */}
      {endSlot && (
        <div className="hidden md:flex items-center gap-4 lg:gap-[29px]">
          {endSlot}
        </div>
      )}
    </div>
  );
}
