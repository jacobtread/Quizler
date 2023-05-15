import { describe, expect, it } from "vitest";
import { deepCopy, getNumberWithOrdinal } from "./utils";

describe("Deep Copy", () => {
  it("Should produce a copy of the provided object", () => {
    const input = { a: "Test", b: { a: 1, arr: [1, 2, 3], value: {} } };
    const copy = deepCopy(input);

    // Copy should be equal
    expect(copy).toEqual(input);
    // References should NOT be equal
    expect(copy).not.toBe(input);
  });

  it("Should produce a copy of the provided array", () => {
    const input = [1, 2, 3];
    const copy = deepCopy(input);

    // Copy should be equal
    expect(copy).toEqual(input);
    // References should NOT be equal
    expect(copy).not.toBe(input);
  });
});

describe("Number ordinals", () => {
  it("Should produce number with correct suffix", () => {
    const input = [1, 2, 3, 4];
    const expected = ["1st", "2nd", "3rd", "4th"];

    for (let i = 0; i < input.length; i++) {
      const a = getNumberWithOrdinal(input[i]);
      const b = expected[i];

      expect(a).toEqual(b);
    }
  });
});
