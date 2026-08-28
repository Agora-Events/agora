/**
 * Inline SVG icon components.
 *
 * Each icon is a pure SVG functional component whose strokes and fills are set
 * to `currentColor`, allowing them to inherit any Tailwind text-colour class
 * (e.g. `text-primary`, `text-white`) from a parent element.
 *
 * Usage:
 *   <ChevronDown className="text-primary" size={20} />
 */

interface IconProps {
  size?: number;
  className?: string;
}

export function ChevronDown({ size = 24, className }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <polyline points="6 9 12 15 18 9" />
    </svg>
  );
}

export function ChevronUp({ size = 24, className }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <polyline points="18 15 12 9 6 15" />
    </svg>
  );
}

export function Camera({ size = 24, className }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <path d="M4 19V9C4 7.89543 4.89543 7 6 7H7.17157C7.70201 7 8.21071 6.78929 8.58579 6.41421L10.4142 4.58579C10.7893 4.21071 11.298 4 11.8284 4H12.1716C12.702 4 13.2107 4.21071 13.5858 4.58579L15.4142 6.41421C15.7893 6.78929 16.298 7 16.8284 7H18C19.1046 7 20 7.89543 20 9V19C20 20.1046 19.1046 21 18 21H6C4.89543 21 4 20.1046 4 19Z" />
      <path d="M12 17C14.2091 17 16 15.2091 16 13C16 10.7909 14.2091 9 12 9C9.79086 9 8 10.7909 8 13C8 15.2091 9.79086 17 12 17Z" />
    </svg>
  );
}

export function CheckCircle2({ size = 24, className }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <circle cx="12" cy="12" r="10" />
      <path d="m9 12 2 2 4-4" />
    </svg>
  );
}

export function Home({ size = 24, className }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      stroke="currentColor"
      strokeWidth={1.5}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <path d="M3 11.9896V14.5C3 17.7998 3 19.4497 4.02513 20.4749C5.05025 21.5 6.70017 21.5 10 21.5H14C17.2998 21.5 18.9497 21.5 19.9749 20.4749C21 19.4497 21 17.7998 21 14.5V11.9896C21 10.3083 21 9.46773 20.6441 8.74005C20.2882 8.01237 19.6247 7.49628 18.2976 6.46411L16.2976 4.90855C14.2331 3.30285 13.2009 2.5 12 2.5C10.7991 2.5 9.76689 3.30285 7.70242 4.90855L5.70241 6.46411C4.37533 7.49628 3.71179 8.01237 3.3559 8.74005C3 9.46773 3 10.3083 3 11.9896Z" />
      <path d="M15 17C14.2005 17.6224 13.1502 18 12 18C10.8497 18 9.79953 17.6224 9 17" />
    </svg>
  );
}

export function ExternalLink({ size = 24, className }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
      <polyline points="15 3 21 3 21 9" />
      <line x1="10" y1="14" x2="21" y2="3" />
    </svg>
  );
}

export function Share2({ size = 24, className }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <circle cx="18" cy="5" r="3" />
      <circle cx="6" cy="12" r="3" />
      <circle cx="18" cy="19" r="3" />
      <line x1="8.59" y1="13.51" x2="15.42" y2="17.49" />
      <line x1="15.41" y1="6.51" x2="8.59" y2="10.49" />
    </svg>
  );
}

export function X({ size = 24, className }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <line x1="18" y1="6" x2="6" y2="18" />
      <line x1="6" y1="6" x2="18" y2="18" />
    </svg>
  );
}

export function Minus({ size = 24, className }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <line x1="5" y1="12" x2="19" y2="12" />
    </svg>
  );
}

export function Plus({ size = 24, className }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <line x1="12" y1="5" x2="12" y2="19" />
      <line x1="5" y1="12" x2="19" y2="12" />
    </svg>
  );
}

export function Ticket({ size = 24, className }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      stroke="currentColor"
      strokeWidth={1.5}
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <path d="M2.46433 9.34375C2.21579 9.34375 1.98899 9.14229 2.00041 8.87895C2.06733 7.33687 2.25481 6.33298 2.78008 5.53884C3.08228 5.08196 3.45765 4.68459 3.88923 4.36468C5.05575 3.5 6.70139 3.5 9.99266 3.5H14.0074C17.2986 3.5 18.9443 3.5 20.1108 4.36468C20.5424 4.68459 20.9177 5.08196 21.2199 5.53884C21.7452 6.33289 21.9327 7.33665 21.9996 8.87843C22.011 9.14208 21.7839 9.34375 21.5351 9.34375C20.1493 9.34375 19.0259 10.533 19.0259 12C19.0259 13.467 20.1493 14.6562 21.5351 14.6562C21.7839 14.6562 22.011 14.8579 21.9996 15.1216C21.9327 16.6634 21.7452 17.6671 21.2199 18.4612C20.9177 18.918 20.5424 19.3154 20.1108 19.6353C18.9443 20.5 17.2986 20.5 14.0074 20.5H9.99266C6.70139 20.5 5.05575 20.5 3.88923 19.6353C3.45765 19.3154 3.08228 18.918 2.78008 18.4612C2.25481 17.667 2.06733 16.6631 2.00041 15.1211C1.98899 14.8577 2.21579 14.6562 2.46433 14.6562C3.85012 14.6562 4.97352 13.467 4.97352 12C4.97352 10.533 3.85012 9.34375 2.46433 9.34375Z" />
      <path d="M9 3.5V20.5" strokeLinecap="round" />
    </svg>
  );
}

export function ArrowRight({ size = 24, className }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <path d="M18.5 12.002H5" />
      <path d="M13.0001 18.002C13.0001 18.002 19 13.583 19 12.0019C19 10.4208 13 6.00195 13 6.00195" />
    </svg>
  );
}

export function Gift({ size = 24, className }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      <polyline points="20 12 20 22 4 22 4 12" />
      <rect x="2" y="7" width="20" height="5" />
      <line x1="12" y1="22" x2="12" y2="7" />
      <path d="M12 7H7.5a2.5 2.5 0 0 1 0-5C11 2 12 7 12 7z" />
      <path d="M12 7h4.5a2.5 2.5 0 0 0 0-5C13 2 12 7 12 7z" />
    </svg>
  );
}
