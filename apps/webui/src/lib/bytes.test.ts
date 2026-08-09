import { describe, expect, it } from "vitest";
import { formatBytes } from "./bytes";

describe("formatBytes", () => {
  it("normalizes invalid and negative values", () => {
    expect(formatBytes(Number.NaN)).toBe("0 B");
    expect(formatBytes(Number.POSITIVE_INFINITY)).toBe("0 B");
    expect(formatBytes(-1)).toBe("0 B");
  });

  it("leaves byte-sized values unscaled", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1023)).toBe("1023 B");
  });

  it("scales IEC units", () => {
    expect(formatBytes(1024)).toBe("1.0 KiB");
    expect(formatBytes(5.5 * 1024 * 1024)).toBe("5.5 MiB");
    expect(formatBytes(3 * 1024 ** 4)).toBe("3.0 TiB");
  });

  it("can use metric labels", () => {
    expect(formatBytes(1024, "metric")).toBe("1.0 KB");
    expect(formatBytes(2 * 1024 ** 3, "metric")).toBe("2.0 GB");
  });
});
