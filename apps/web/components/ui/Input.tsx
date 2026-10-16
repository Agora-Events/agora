import React from "react";

/**
 * Placeholder colours that meet WCAG AA (4.5:1) against the background each
 * tone sits on (#1576). The previous `gray-400` on white measured 2.6:1 and
 * `ink-deep/40` on the warm surface 2.5:1 (2.3:1 in dark mode).
 *
 * - `light`:   gray-500 on white                          → 4.84:1
 * - `surface`: ink-deep/80 on surface, light / dark theme → 9.02:1 / 5.04:1
 */
export const PLACEHOLDER_TONE_CLASS = {
  light: "placeholder:text-gray-500",
  surface: "placeholder:text-ink-deep/80",
} as const;

export type InputTone = keyof typeof PLACEHOLDER_TONE_CLASS;

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  /** Background the input sits on; picks a placeholder colour that passes AA. */
  tone?: InputTone;
}

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  function Input({ tone = "light", className = "", ...props }, ref) {
    return (
      <input
        ref={ref}
        className={`${PLACEHOLDER_TONE_CLASS[tone]} ${className}`.trim()}
        {...props}
      />
    );
  },
);
