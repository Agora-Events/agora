import React from "react";
import Image from "next/image";

/**
 * Props for the Button component
 * @interface ButtonProps
 */
interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /** Visual style variant of the button */
  variant?: "primary" | "secondary";
  /** Shadow color for the button's drop shadow effect */
  shadowColor?: string;
  /** Text color class or custom color */
  textColor?: string;
  /** Background color class or custom color */
  backgroundColor?: string;
  /** Content to render inside the button */
  children: React.ReactNode;
  /** When true, shows a spinner and reduces label opacity */
  isLoading?: boolean;
}

/**
 * Custom Button component with distinctive shadow and hover effects
 *
 * Features:
 * - Custom shadow effect that moves on hover
 * - Active state with deeper shadow translation
 * - Support for both Tailwind and custom colors
 * - Fully accessible with all standard button props
 * - Loading state with spinner overlay (no layout shift)
 *
 * @param props - ButtonProps containing button configuration
 * @returns React component that renders a styled button
 */
export function Button({
  className = "",
  shadowColor = "rgba(0,0,0,1)",
  textColor = "text-black",
  backgroundColor = "bg-white",
  children,
  style,
  isLoading = false,
  disabled,
  ...props
}: ButtonProps) {
  const isTailwindBg = backgroundColor.startsWith("bg-");
  const isTailwindText = textColor.startsWith("text-");

  const bgClass = isTailwindBg ? backgroundColor : "";
  const textClass = isTailwindText ? textColor : "";

  const customStyle: React.CSSProperties = {
    ...style,
    backgroundColor: !isTailwindBg ? backgroundColor : undefined,
    color: !isTailwindText ? textColor : undefined,
    boxShadow: `-4px 4px 0px 0px ${shadowColor}`,
  };

  return (
    <button
      className={`
        relative group flex items-center justify-center gap-2 px-6 py-3 rounded-full border border-black
        font-semibold transition-all whitespace-nowrap
        hover:-translate-x-[2px] hover:translate-y-[2px]
        hover:shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)]
        active:-translate-x-[4px] active:translate-y-[4px] active:shadow-none
        ${bgClass} ${textClass} ${className}
      `}
      style={customStyle}
      disabled={isLoading || disabled}
      aria-busy={isLoading}
      {...props}
    >
      {isLoading && (
        <Image
          src="/icons/spinner.svg"
          alt="Loading"
          width={18}
          height={18}
          className="absolute inset-0 m-auto animate-spin"
          aria-hidden="true"
        />
      )}
      <span className={isLoading ? "opacity-40" : undefined}>
        {children}
      </span>
    </button>
  );
}
