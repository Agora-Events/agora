export interface HighlightTextProps {
  text: string;
  query: string;
}

function escapeRegExp(input: string): string {
  return input.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/**
 * HighlightText — highlights all occurrences of `query` within `text`
 * using `<mark>` with the design-system accent background.
 *
 * When `query` is empty or whitespace-only, the original text is returned
 * without any highlighting.
 */
export function HighlightText({ text, query }: HighlightTextProps) {
  const q = query.trim();
  if (!q) return <>{text}</>;

  const escaped = escapeRegExp(q);
  const matcher = new RegExp(escaped, "gi");
  const nodes: React.ReactNode[] = [];
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  let key = 0;

  while ((match = matcher.exec(text)) !== null) {
    if (match.index > lastIndex) {
      nodes.push(text.slice(lastIndex, match.index));
    }
    nodes.push(
      <mark
        key={key++}
        className="bg-accent text-ink px-0.5 rounded-[2px]"
      >
        {match[0]}
      </mark>
    );
    lastIndex = match.index + match[0].length;
    if (match[0].length === 0) {
      matcher.lastIndex++;
    }
  }

  if (lastIndex < text.length) {
    nodes.push(text.slice(lastIndex));
  }

  return <>{nodes}</>;
}
