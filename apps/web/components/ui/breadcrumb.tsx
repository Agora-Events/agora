import Link from "next/link";

/** A single entry in a breadcrumb trail. */
export interface BreadcrumbItem {
  /** Visible text for this level. */
  label: string;
  /** Destination. Omit for the current page (rendered as plain text). */
  href?: string;
}

interface BreadcrumbProps {
  /** Ordered trail, from the root down to the current page. */
  items: BreadcrumbItem[];
  /** Extra classes for the wrapping `<nav>`. */
  className?: string;
  /**
   * Colour classes for the linked ancestors, the separators and the current
   * page. Override when the breadcrumb sits on a surface the default tokens
   * were not chosen for — e.g. the Help Center's accent banner.
   */
  linkClassName?: string;
  separatorClassName?: string;
  currentClassName?: string;
}

/**
 * Hierarchical breadcrumb navigation.
 *
 * Renders an ordered list inside a labelled `<nav>`; the final entry is marked
 * with `aria-current="page"` and is never a link, matching the WAI-ARIA
 * breadcrumb pattern. Separators are decorative and hidden from screen readers.
 *
 * @example
 * <Breadcrumb
 *   items={[
 *     { label: "Home", href: "/" },
 *     { label: "Discover", href: "/discover" },
 *     { label: event.title },
 *   ]}
 * />
 */
export function Breadcrumb({
  items,
  className = "",
  linkClassName = "text-ink-soft/70 hover:text-ink-soft",
  separatorClassName = "text-ink-soft/40",
  currentClassName = "text-ink-soft",
}: BreadcrumbProps) {
  if (items.length === 0) return null;

  return (
    <nav aria-label="breadcrumb" className={className}>
      <ol className="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm">
        {items.map((item, index) => {
          const isLast = index === items.length - 1;

          return (
            <li
              key={`${item.href ?? "current"}-${item.label}`}
              className="flex items-center gap-2 min-w-0"
            >
              {isLast || !item.href ? (
                <span
                  aria-current={isLast ? "page" : undefined}
                  className={`max-w-[16rem] truncate font-semibold ${currentClassName}`}
                  title={item.label}
                >
                  {item.label}
                </span>
              ) : (
                <Link
                  href={item.href}
                  className={`rounded-sm transition-colors hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent ${linkClassName}`}
                >
                  {item.label}
                </Link>
              )}

              {!isLast && (
                <span
                  aria-hidden="true"
                  className={`select-none ${separatorClassName}`}
                >
                  /
                </span>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}

export default Breadcrumb;
