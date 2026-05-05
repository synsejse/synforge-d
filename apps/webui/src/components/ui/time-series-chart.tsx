import { useMemo } from "react";
import {
  Area,
  CartesianGrid,
  Legend,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
  AreaChart,
} from "recharts";
import type { TimeSeriesPoint, TimeSeriesResponse } from "../../lib/types";
import LoadingBlock from "./loading-block";

interface Props {
  data: TimeSeriesResponse | undefined;
  isLoading?: boolean;
  height?: number;
  succeededColor?: string;
  failedColor?: string;
  emptyLabel?: string;
}

interface ChartRow {
  ts: number;
  label: string;
  succeeded: number;
  failed: number;
}

export default function TimeSeriesChart({
  data,
  isLoading = false,
  height = 240,
  succeededColor = "var(--theme-terminal-green)",
  failedColor = "var(--theme-error-red)",
  emptyLabel = "No activity in this window.",
}: Props) {
  const range = data?.range ?? "24h";

  const rows = useMemo<ChartRow[]>(() => {
    if (!data) return [];
    return data.points.map((point: TimeSeriesPoint) => {
      const ts = Date.parse(point.timestamp);
      return {
        ts,
        label: formatBucketLabel(ts, range),
        succeeded: point.succeeded,
        failed: point.failed,
      };
    });
  }, [data, range]);

  const totalEvents = rows.reduce((sum, r) => sum + r.succeeded + r.failed, 0);

  if (isLoading) {
    return (
      <div style={{ height }}>
        <LoadingBlock label="Loading chart…" lines={0} />
      </div>
    );
  }

  if (!data || rows.length === 0 || totalEvents === 0) {
    return (
      <div
        className="flex items-center justify-center border-2 border-dashed border-edge-strong bg-surface-alt p-8 font-mono text-xs uppercase tracking-[0.18em] text-soft"
        style={{ height }}
      >
        {emptyLabel}
      </div>
    );
  }

  return (
    <div style={{ width: "100%", height }}>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart
          data={rows}
          margin={{ top: 8, right: 8, left: 0, bottom: 0 }}
        >
          <CartesianGrid
            stroke="var(--theme-border)"
            strokeDasharray="2 4"
            vertical={false}
          />
          <XAxis
            dataKey="label"
            stroke="var(--theme-text-soft)"
            tick={{ fontSize: 10, fontFamily: "Fira Code, monospace" }}
            tickLine={false}
            axisLine={{ stroke: "var(--theme-border-strong)" }}
            minTickGap={28}
          />
          <YAxis
            stroke="var(--theme-text-soft)"
            tick={{ fontSize: 10, fontFamily: "Fira Code, monospace" }}
            tickLine={false}
            axisLine={{ stroke: "var(--theme-border-strong)" }}
            allowDecimals={false}
            width={32}
          />
          <Tooltip
            cursor={{ fill: "rgba(255, 255, 255, 0.04)" }}
            contentStyle={{
              backgroundColor: "var(--theme-surface)",
              border: "2px solid var(--theme-border-strong)",
              borderRadius: 0,
              fontFamily: "Fira Code, monospace",
              fontSize: 12,
              padding: "0.5rem 0.75rem",
            }}
            labelStyle={{
              color: "var(--theme-text-strong)",
              textTransform: "uppercase",
              letterSpacing: "0.12em",
              fontWeight: 700,
              marginBottom: 4,
            }}
            itemStyle={{ color: "var(--theme-text-muted)" }}
          />
          <Legend
            wrapperStyle={{
              fontFamily: "Fira Code, monospace",
              fontSize: 11,
              textTransform: "uppercase",
              letterSpacing: "0.12em",
              color: "var(--theme-text-muted)",
            }}
            iconType="square"
          />
          <Area
            type="stepAfter"
            dataKey="succeeded"
            stackId="status"
            stroke={succeededColor}
            fill={succeededColor}
            fillOpacity={0.55}
            isAnimationActive={false}
          />
          <Area
            type="stepAfter"
            dataKey="failed"
            stackId="status"
            stroke={failedColor}
            fill={failedColor}
            fillOpacity={0.7}
            isAnimationActive={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}

function formatBucketLabel(ts: number, range: string): string {
  const d = new Date(ts);
  if (range === "30d") {
    return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  }
  if (range === "7d") {
    return d.toLocaleString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      hour12: false,
    });
  }
  return d.toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
}
