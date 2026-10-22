import { describe, expect, it } from "vitest";
import { matchScore } from "./fuzzy";

describe("matchScore", () => {
  it("matches letters in order and rejects missing ones", () => {
    expect(matchScore("Competitive", "cmp")).not.toBeNull();
    expect(matchScore("Competitive", "xyz")).toBeNull();
  });

  it("ranks a tighter, earlier match first", () => {
    const tight = matchScore("Store", "sto") ?? Infinity;
    const loose = matchScore("Swiftplay mode", "sto") ?? Infinity;
    expect(tight).toBeLessThan(loose);
  });

  it("treats an empty query as matching everything equally", () => {
    expect(matchScore("Anything", "  ")).toBe(0);
  });
});
