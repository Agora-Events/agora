const MAX_SLUG_LENGTH = 80;
const SUFFIX_LENGTH = 6;

/**
 * Converts a title into a URL-safe slug: lowercase, diacritics stripped,
 * non-alphanumeric runs collapsed into single hyphens, trimmed to 80 characters.
 */
export function slugify(title: string): string {
  const slug = title
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, MAX_SLUG_LENGTH)
    .replace(/-+$/g, "");

  return slug || "event";
}

/**
 * Appends a short random suffix to a slug, used to recover from a unique
 * constraint collision instead of failing the insert.
 */
export function withRandomSuffix(slug: string): string {
  const suffix = Math.random().toString(36).slice(2, 2 + SUFFIX_LENGTH);
  const base = slug.slice(0, MAX_SLUG_LENGTH - SUFFIX_LENGTH - 1).replace(/-+$/g, "");
  return `${base}-${suffix}`;
}
