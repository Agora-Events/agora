import fs from "fs";
import path from "path";
import { describe, expect, it } from "vitest";

/**
 * Recursively flattens a nested plain object into sorted, dotted key paths.
 * Array values and null/primitives are treated as leaf values.
 */
export function flattenKeys(obj: unknown, prefix = ""): string[] {
  if (typeof obj !== "object" || obj === null || Array.isArray(obj)) {
    return prefix ? [prefix] : [];
  }

  const keys: string[] = [];
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (typeof value === "object" && value !== null && !Array.isArray(value)) {
      keys.push(...flattenKeys(value, fullKey));
    } else {
      keys.push(fullKey);
    }
  }

  return keys.sort();
}

export interface KeyDiff {
  missing: string[];
  extra: string[];
}

/**
 * Compares target locale keys against baseline (en) keys.
 */
export function diffKeys(baseKeys: string[], targetKeys: string[]): KeyDiff {
  const baseSet = new Set(baseKeys);
  const targetSet = new Set(targetKeys);

  const missing = baseKeys.filter((k) => !targetSet.has(k)).sort();
  const extra = targetKeys.filter((k) => !baseSet.has(k)).sort();

  return { missing, extra };
}

/**
 * Formats a detailed failure message listing missing and extra keys per locale.
 */
export function formatDiffMessage(locale: string, diff: KeyDiff): string {
  const parts: string[] = [];
  if (diff.missing.length > 0) {
    parts.push(
      `Missing keys (${diff.missing.length}):\n${diff.missing.map((k) => `  - ${k}`).join("\n")}`
    );
  }
  if (diff.extra.length > 0) {
    parts.push(
      `Extra keys (${diff.extra.length}):\n${diff.extra.map((k) => `  + ${k}`).join("\n")}`
    );
  }
  return `Locale "${locale}" translation keys mismatch against en.json:\n${parts.join("\n\n")}`;
}

describe("translation key utilities", () => {
  it("flattens nested plain objects cleanly and sorts output keys deterministically", () => {
    const input = {
      z: "last",
      a: {
        d: "nested-d",
        b: "nested-b",
      },
      list: ["one", "two"],
      empty: null,
    };

    expect(flattenKeys(input)).toEqual(["a.b", "a.d", "empty", "list", "z"]);
  });

  it("detects missing keys and formats the failure message", () => {
    const base = ["common.cancel", "common.ok", "footer.copyright"];
    const target = ["common.ok"];
    const diff = diffKeys(base, target);

    expect(diff.missing).toEqual(["common.cancel", "footer.copyright"]);
    expect(diff.extra).toEqual([]);

    const message = formatDiffMessage("es", diff);
    expect(message).toContain('Locale "es" translation keys mismatch against en.json:');
    expect(message).toContain("Missing keys (2):");
    expect(message).toContain("  - common.cancel");
    expect(message).toContain("  - footer.copyright");
    expect(message).not.toContain("Extra keys");
  });

  it("detects extra keys and formats the failure message", () => {
    const base = ["common.ok"];
    const target = ["admin.dashboard", "common.extra", "common.ok"];
    const diff = diffKeys(base, target);

    expect(diff.missing).toEqual([]);
    expect(diff.extra).toEqual(["admin.dashboard", "common.extra"]);

    const message = formatDiffMessage("fr", diff);
    expect(message).toContain('Locale "fr" translation keys mismatch against en.json:');
    expect(message).toContain("Extra keys (2):");
    expect(message).toContain("  + admin.dashboard");
    expect(message).toContain("  + common.extra");
    expect(message).not.toContain("Missing keys");
  });

  it("detects both missing and extra keys together", () => {
    const base = ["home.title", "home.subtitle"];
    const target = ["home.legacy", "home.title"];
    const diff = diffKeys(base, target);

    expect(diff.missing).toEqual(["home.subtitle"]);
    expect(diff.extra).toEqual(["home.legacy"]);

    const message = formatDiffMessage("de", diff);
    expect(message).toContain("Missing keys (1):");
    expect(message).toContain("  - home.subtitle");
    expect(message).toContain("Extra keys (1):");
    expect(message).toContain("  + home.legacy");
  });
});

describe("translation keys sync across locales in apps/web/messages", () => {
  const messagesDir = path.resolve(__dirname, "../messages");
  const allJsonFiles = fs.existsSync(messagesDir)
    ? fs
        .readdirSync(messagesDir)
        .filter((file) => file.endsWith(".json"))
        .sort()
    : [];

  it("resolves messages directory and discovers non-en locales without passing vacuously", () => {
    expect(fs.existsSync(messagesDir)).toBe(true);
    expect(allJsonFiles).toContain("en.json");

    const nonEnLocales = allJsonFiles.filter((file) => file !== "en.json");
    expect(nonEnLocales.length).toBeGreaterThan(0);
    expect(nonEnLocales).toContain("es.json");
    expect(nonEnLocales).toContain("fr.json");
  });

  const nonEnFiles = allJsonFiles.filter((file) => file !== "en.json");

  for (const file of nonEnFiles) {
    const locale = path.basename(file, ".json");

    it(`asserts ${file} has exactly the same keys as en.json`, () => {
      const enPath = path.join(messagesDir, "en.json");
      const enContent = JSON.parse(fs.readFileSync(enPath, "utf-8"));
      const enKeys = flattenKeys(enContent);

      const filePath = path.join(messagesDir, file);
      const content = JSON.parse(fs.readFileSync(filePath, "utf-8"));
      const localeKeys = flattenKeys(content);

      const diff = diffKeys(enKeys, localeKeys);
      const errorMessage = formatDiffMessage(locale, diff);

      expect(diff.missing, errorMessage).toEqual([]);
      expect(diff.extra, errorMessage).toEqual([]);
      expect(localeKeys, errorMessage).toEqual(enKeys);
    });
  }
});
