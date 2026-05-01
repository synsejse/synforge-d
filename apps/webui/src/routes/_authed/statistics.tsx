import { createFileRoute } from "@tanstack/react-router";
import StatisticsPage from "../../features/statistics/statistics-page";
import type { TimeRange } from "../../lib/types";

interface StatisticsSearch {
  range?: TimeRange;
}

const VALID_RANGES: ReadonlySet<TimeRange> = new Set(["24h", "7d", "30d"]);

export const Route = createFileRoute("/_authed/statistics")({
  validateSearch: (search: Record<string, unknown>): StatisticsSearch => {
    const raw = typeof search.range === "string" ? (search.range as TimeRange) : undefined;
    return { range: raw && VALID_RANGES.has(raw) ? raw : undefined };
  },
  component: StatisticsPage,
});
