/**
 * useThemeColor
 *
 * Returns a single resolved color value for the active theme.
 * Reads the scheme from ThemeContext so it reacts instantly to both
 * system-level Appearance changes and manual overrides.
 *
 * Usage:
 *   const bg = useThemeColor({ light: '#FFF', dark: '#000' }, 'background');
 */

import { Colors } from '@/constants/Colors';
import { useThemeContext } from '@/context/ThemeContext';

export function useThemeColor(
  props: { light?: string; dark?: string },
  colorName: keyof typeof Colors.light & keyof typeof Colors.dark
) {
  const { colorScheme } = useThemeContext();
  const colorFromProps = props[colorScheme];

  return colorFromProps ?? Colors[colorScheme][colorName];
}
