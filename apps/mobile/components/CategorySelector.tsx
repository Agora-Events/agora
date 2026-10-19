import React from "react";
import { ScrollView, StyleSheet, Text, TouchableOpacity } from "react-native";

/**
 * Horizontal category chips for the Discover screen (issue #1014).
 */

export const EVENT_CATEGORIES = [
  "Music",
  "Tech",
  "Sports",
  "Arts",
  "Business",
] as const;

export type EventCategory = (typeof EVENT_CATEGORIES)[number];

export interface CategorySelectorProps {
  /** Currently selected category, or null for "all". */
  selected: string | null;
  /** Called with the category id, or null when the active chip is tapped again. */
  onSelect: (category: string | null) => void;
  categories?: readonly string[];
  testID?: string;
}

export function CategorySelector({
  selected,
  onSelect,
  categories = EVENT_CATEGORIES,
  testID,
}: CategorySelectorProps) {
  return (
    <ScrollView
      horizontal
      showsHorizontalScrollIndicator={false}
      // Chips are a fixed, short list, so keeping them mounted avoids the
      // blank-cell flicker virtualisation causes on fast horizontal scrolls.
      removeClippedSubviews={false}
      contentContainerStyle={styles.content}
      testID={testID ?? "category-selector"}
    >
      {categories.map((category) => {
        const isActive = selected === category;
        return (
          <TouchableOpacity
            key={category}
            testID={`category-chip-${category}`}
            accessibilityRole="button"
            // Communicates the toggle state to screen readers; without it a
            // selected chip is indistinguishable from an unselected one.
            accessibilityState={{ selected: isActive }}
            accessibilityLabel={`${category} category`}
            // Tapping the active chip clears the filter, so there is a way back
            // to "all" without hunting for a separate reset control.
            onPress={() => onSelect(isActive ? null : category)}
            style={[styles.chip, isActive ? styles.chipActive : styles.chipInactive]}
          >
            <Text style={[styles.label, isActive ? styles.labelActive : styles.labelInactive]}>
              {category}
            </Text>
          </TouchableOpacity>
        );
      })}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  content: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    gap: 8,
  },
  chip: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 999,
    borderWidth: 1,
    // Chips must not shrink to fit; a squeezed chip truncates its label.
    flexShrink: 0,
  },
  chipActive: {
    backgroundColor: "#FACC15",
    borderColor: "#FACC15",
  },
  chipInactive: {
    backgroundColor: "transparent",
    borderColor: "#3F3F46",
  },
  label: {
    fontSize: 14,
    fontWeight: "600",
  },
  labelActive: {
    color: "#000000",
  },
  labelInactive: {
    color: "#FFFFFF",
  },
});

export default CategorySelector;
