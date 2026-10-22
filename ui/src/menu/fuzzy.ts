// Fuzzy matching for the command palette, kept pure so it can be tested on its own.

/** Subsequence match: every typed letter in order. Earlier and tighter matches rank higher. */
export function matchScore(text: string, query: string): number | null {
  const haystack = text.toLowerCase();
  const needle = query.toLowerCase().replace(/\s+/g, "");
  if (!needle) return 0;
  let position = -1;
  let score = 0;
  for (const char of needle) {
    const next = haystack.indexOf(char, position + 1);
    if (next === -1) return null;
    score += next - position - 1;
    position = next;
  }
  return score + haystack.indexOf(needle[0] ?? "") * 0.5;
}
