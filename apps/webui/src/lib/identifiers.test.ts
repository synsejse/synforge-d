import { describe, expect, it } from "vitest";
import { formatCompactId } from "./identifiers";

describe("formatCompactId", () => {
  it("shortens long identifiers while keeping both distinguishing ends", () => {
    expect(formatCompactId("8f6e4bc1-4814-4cd9-a8ce-f24bad4d4d6b")).toBe(
      "8f6e4bc1…4d6b",
    );
  });

  it("leaves short identifiers intact", () => {
    expect(formatCompactId("sync-42")).toBe("sync-42");
  });
});
