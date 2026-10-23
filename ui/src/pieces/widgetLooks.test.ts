import { describe, expect, it } from "vitest";
import { autoLayout } from "./widgetLooks";

describe("autoLayout", () => {
  it("stacks a card-shaped box", () => {
    expect(autoLayout(320, 140)).toBe("stack");
  });

  it("lays a wide, short strip out as a row", () => {
    expect(autoLayout(520, 120)).toBe("row");
  });

  it("drops to compact when either side is tiny", () => {
    expect(autoLayout(400, 40)).toBe("compact");
    expect(autoLayout(150, 150)).toBe("compact");
  });
});
