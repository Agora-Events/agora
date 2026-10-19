/**
 * ThemeContext
 *
 * Tracks the device color scheme (light | dark) via React Native's
 * Appearance API and re-renders consumers whenever the system theme changes.
 *
 * Exposes:
 *   colorScheme  – 'light' | 'dark' (never null)
 *   theme        – resolved color tokens for the current scheme
 *   toggleTheme  – manual override (useful for in-app toggle / testing)
 */

import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from 'react';
import { Appearance, ColorSchemeName } from 'react-native';
import { Colors } from '@/constants/Colors';

// ─── Types ────────────────────────────────────────────────────────────────────

export type ThemeColorScheme = 'light' | 'dark';

export type ThemeColors = typeof Colors.light; // { text, background, tint, icon, … }

export interface ThemeContextValue {
  /** Resolved color scheme – always 'light' or 'dark'. */
  colorScheme: ThemeColorScheme;
  /** Resolved color token map for the active scheme. */
  theme: ThemeColors;
  /** Brand palette (scheme-independent tokens). */
  palette: Omit<typeof Colors, 'light' | 'dark'>;
  /** Override the scheme manually. Pass `null` to revert to system default. */
  toggleTheme: (scheme?: ThemeColorScheme | null) => void;
}

// ─── Context ──────────────────────────────────────────────────────────────────

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);
ThemeContext.displayName = 'ThemeContext';

// ─── Helpers ─────────────────────────────────────────────────────────────────

function resolveScheme(raw: ColorSchemeName): ThemeColorScheme {
  return raw === 'light' ? 'light' : 'dark';
}

// Brand tokens that are not scheme-dependent
const { light: _light, dark: _dark, ...palette } = Colors;

// ─── Provider ─────────────────────────────────────────────────────────────────

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  // Seed from the current system value so there's no flash on first render
  const [colorScheme, setColorScheme] = useState<ThemeColorScheme>(() =>
    resolveScheme(Appearance.getColorScheme())
  );

  // Whether the user manually pinned a scheme (overrides system changes)
  const [pinned, setPinned] = useState<ThemeColorScheme | null>(null);

  useEffect(() => {
    // Appearance.addChangeListener is available since RN 0.62
    const subscription = Appearance.addChangeListener(({ colorScheme: next }) => {
      // Only apply system changes when not manually overridden
      if (pinned === null) {
        setColorScheme(resolveScheme(next));
      }
    });

    return () => subscription.remove();
  }, [pinned]);

  const toggleTheme = useCallback((scheme?: ThemeColorScheme | null) => {
    if (scheme == null) {
      // Revert to system theme
      setPinned(null);
      setColorScheme(resolveScheme(Appearance.getColorScheme()));
    } else {
      setPinned(scheme);
      setColorScheme(scheme);
    }
  }, []);

  const theme = useMemo<ThemeColors>(() => Colors[colorScheme], [colorScheme]);

  const value = useMemo<ThemeContextValue>(
    () => ({ colorScheme, theme, palette, toggleTheme }),
    [colorScheme, theme, toggleTheme]
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

// ─── Internal helper (used by useTheme & useThemeColor) ───────────────────────

export function useThemeContext(): ThemeContextValue {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    throw new Error('useThemeContext must be used within a <ThemeProvider>.');
  }
  return ctx;
}

export default ThemeContext;
