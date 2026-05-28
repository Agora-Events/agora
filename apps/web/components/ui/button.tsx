import React from "react";

/**
 * Variant → bg / text / shadow presets.
 * - primary  : yellow fill, black text  (main CTA)
 * - secondary: white fill, black text   (default)
 * - dark     : black fill, white text
 * - ghost    : transparent, black text, no shadow
 */
const VARIANTS = {
  primary: {
    bg: "bg-[#FDDA23]",
    text: "text-black",
    shadow: "rgba(0,0,0,1)",
  },
  secondary: {
    bg: "bg-white",
    text: "text-black",
    shadow: "rgba(0,0,0,1)",
  },
  dark: {
    bg: "bg-black",
    text: "text-white",
    shadow: "rgba(0,0,0,0.4)",
  },
  ghost: {
    bg: "bg-transparent",
    text: "text-black",
    shadow: "transparent",
  },
} as const;

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: keyof typeof VARIANTS;
  /** Override shadow color (ignored when variant is set unless also passed) */
  shadowColor?: string;
  /** Override text color class */
  textColor?: string;
  /** Override background color class */
  backgroundColor?: string;
  children: React.ReactNode;
}

export function Button({
  className = "",
  variant,
  shadowColor,
  textColor,
  backgroundColor,
  children,
  style,
  ...props
}: ButtonProps) {
  const preset = variant ? VARIANTS[variant] : null;

  const bg = backgroundColor ?? preset?.bg ?? VARIANTS.secondary.bg;
  const text = textColor ?? preset?.text ?? VARIANTS.secondary.text;
  const shadow = shadowColor ?? preset?.shadow ?? VARIANTS.secondary.shadow;

  const isTailwindBg = bg.startsWith("bg-");
  const isTailwindText = text.startsWith("text-");

  const customStyle: React.CSSProperties = {
    ...style,
    backgroundColor: !isTailwindBg ? bg : undefined,
    color: !isTailwindText ? text : undefined,
    boxShadow: `-4px 4px 0px 0px ${shadow}`,
  };

  return (
    <button
      type="button"
      className={`
        group flex items-center justify-center gap-2 px-6 py-3 rounded-full border border-black
        font-semibold transition-all whitespace-nowrap
        hover:-translate-x-[2px] hover:translate-y-[2px]
        hover:shadow-[-2px_2px_0px_0px_rgba(0,0,0,1)]
        active:-translate-x-[4px] active:translate-y-[4px] active:shadow-none
        ${isTailwindBg ? bg : ""} ${isTailwindText ? text : ""} ${className}
      `}
      style={customStyle}
      {...props}
    >
      {children}
    </button>
  );
}
