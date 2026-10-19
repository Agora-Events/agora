/**
 * useTheme
 *
 * Convenience hook that exposes the full ThemeContext value.
 *
 * Usage:
 *   const { colorScheme, theme, palette, toggleTheme } = useTheme();
 *
 *   // Read a theme token
 *   const bg = theme.background;     // '#0F0F10' (dark) | '#FFFFFF' (light)
 *   const text = theme.text;
 *
 *   // Read a brand token (scheme-independent)
 *   const yellow = palette.primaryYellow;  // '#FDDA23'
 *
 *   // Manual override
 *   toggleTheme('light');   // pin to light
 *   toggleTheme(null);      // revert to system
 */

import { useThemeContext } from '@/context/ThemeContext';

export function useTheme() {
  return useThemeContext();
}

export default useTheme;
