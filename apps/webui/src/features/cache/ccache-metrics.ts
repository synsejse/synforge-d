import type { BuildCcacheStats } from "../../lib/types";

const numberFormatter = new Intl.NumberFormat();

export interface CcacheMetrics {
  hits: number;
  cacheableCalls: number;
  hitRate: number | null;
  temperature: "no-calls" | "cold" | "warming" | "warm" | "hot";
}

export function getCcacheMetrics(stats: BuildCcacheStats): CcacheMetrics {
  const hits = stats.direct_hits + stats.preprocessed_hits;
  const cacheableCalls = hits + stats.cache_misses;
  const hitRate = cacheableCalls > 0 ? (hits / cacheableCalls) * 100 : null;

  return {
    hits,
    cacheableCalls,
    hitRate,
    temperature:
      hitRate == null
        ? "no-calls"
        : hitRate === 0
          ? "cold"
          : hitRate < 40
            ? "warming"
            : hitRate < 80
              ? "warm"
              : "hot",
  };
}

export function formatCcacheCount(value: number): string {
  return numberFormatter.format(value);
}

export function formatCcacheRate(value: number | null): string {
  return value == null ? "—" : `${value.toFixed(1)}%`;
}
