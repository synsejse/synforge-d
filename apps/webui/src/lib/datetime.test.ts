import { afterEach, describe, expect, it, vi } from "vitest";
import {
  compareTimestampsDesc,
  formatDurationBetween,
  formatDurationSeconds,
  formatJobDuration,
  parseTimestamp,
} from "./datetime";

afterEach(() => {
  vi.useRealTimers();
});

describe("parseTimestamp", () => {
  it("parses ISO strings and rejects empty or invalid values", () => {
    expect(parseTimestamp("2025-04-03T12:34:56Z")?.toISOString()).toBe(
      "2025-04-03T12:34:56.000Z",
    );
    expect(parseTimestamp("  ")).toBeNull();
    expect(parseTimestamp("not a timestamp")).toBeNull();
  });

  it("parses chrono arrays with ordinal dates and offsets", () => {
    const parsed = parseTimestamp([2024, 60, 1, 2, 3, 500_000_000, 2, 0, 0]);
    expect(parsed?.toISOString()).toBe("2024-02-28T23:02:03.500Z");
  });

  it("parses supported timestamp records", () => {
    expect(parseTimestamp({ unix_timestamp: 1_700_000_000 })?.getTime()).toBe(
      1_700_000_000_000,
    );
    expect(
      parseTimestamp({ seconds: 1_700_000_000, nanoseconds: 123_000_000 })?.getTime(),
    ).toBe(1_700_000_000_123);
  });
});

describe("duration helpers", () => {
  it("formats fixed intervals and rejects backwards intervals", () => {
    expect(
      formatDurationBetween(
        "2025-01-01T00:00:00Z",
        "2025-01-01T01:02:03Z",
      ),
    ).toBe("1h 2m");
    expect(
      formatDurationBetween(
        "2025-01-01T00:01:00Z",
        "2025-01-01T00:00:00Z",
      ),
    ).toBe("-");
  });

  it("uses the current time for live jobs", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2025-01-01T00:10:00Z"));

    expect(
      formatJobDuration({
        status: "running",
        created_at: "2025-01-01T00:00:00Z",
        started_at: "2025-01-01T00:02:00Z",
      }),
    ).toEqual({ label: "Running", value: "8m 0s" });
  });

  it("formats numeric durations", () => {
    expect(formatDurationSeconds(Number.NaN)).toBe("0s");
    expect(formatDurationSeconds(59.9)).toBe("59s");
    expect(formatDurationSeconds(3_661)).toBe("1h 1m");
  });
});

describe("compareTimestampsDesc", () => {
  it("sorts newer timestamps first", () => {
    const values = ["2025-01-01T00:00:00Z", "2025-03-01T00:00:00Z"];
    expect(values.sort(compareTimestampsDesc)).toEqual([
      "2025-03-01T00:00:00Z",
      "2025-01-01T00:00:00Z",
    ]);
  });
});
