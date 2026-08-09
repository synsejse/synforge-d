import { describe, expect, it } from "vitest";
import { getCcacheMetrics } from "./ccache-metrics";

describe("getCcacheMetrics", () => {
  it("uses only cacheable calls for the hit rate", () => {
    const metrics = getCcacheMetrics({
      compiler_calls: 20,
      direct_hits: 6,
      preprocessed_hits: 2,
      cache_misses: 2,
      uncacheable_calls: 9,
      error_calls: 1,
    });

    expect(metrics.hits).toBe(8);
    expect(metrics.cacheableCalls).toBe(10);
    expect(metrics.hitRate).toBe(80);
    expect(metrics.temperature).toBe("hot");
  });

  it("reports no calls when no cacheable compilations ran", () => {
    const metrics = getCcacheMetrics({
      compiler_calls: 2,
      direct_hits: 0,
      preprocessed_hits: 0,
      cache_misses: 0,
      uncacheable_calls: 2,
      error_calls: 0,
    });

    expect(metrics.hitRate).toBeNull();
    expect(metrics.temperature).toBe("no-calls");
  });
});
