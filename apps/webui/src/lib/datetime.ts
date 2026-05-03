export function parseTimestamp(value: unknown): Date | null {
  if (value == null) {
    return null;
  }

  if (Array.isArray(value)) {
    if (value.length >= 9 && value.every((item) => typeof item === "number")) {
      const [year, ordinal, hour, minute, second, nanosecond, offsetHour, offsetMinute, offsetSecond] = value;
      const base = new Date(Date.UTC(year, 0, 1, hour, minute, second, Math.floor(nanosecond / 1_000_000)));
      base.setUTCDate(base.getUTCDate() + ordinal - 1);
      const offsetSeconds = (offsetHour * 3600) + (offsetMinute * 60) + offsetSecond;
      const parsed = new Date(base.getTime() - offsetSeconds * 1000);
      return Number.isNaN(parsed.getTime()) ? null : parsed;
    }
    return null;
  }

  if (value instanceof Date) {
    return Number.isNaN(value.getTime()) ? null : value;
  }

  if (typeof value === "number") {
    const parsed = new Date(value);
    return Number.isNaN(parsed.getTime()) ? null : parsed;
  }

  if (typeof value !== "string") {
    if (typeof value === "object") {
      const record = value as Record<string, unknown>;
      if (typeof record.unix_timestamp === "number") {
        const parsed = new Date(record.unix_timestamp * 1000);
        return Number.isNaN(parsed.getTime()) ? null : parsed;
      }
      if (typeof record.seconds === "number") {
        const millis =
          record.seconds * 1000 +
          (typeof record.nanoseconds === "number" ? Math.floor(record.nanoseconds / 1_000_000) : 0);
        const parsed = new Date(millis);
        return Number.isNaN(parsed.getTime()) ? null : parsed;
      }
      if (typeof record.datetime === "string") {
        return parseTimestamp(record.datetime);
      }
    }
    return null;
  }

  const raw = value.trim();
  if (!raw) {
    return null;
  }

  const candidates = [
    raw,
    raw.replace(" ", "T"),
    raw.endsWith("Z") ? raw : `${raw}Z`,
    raw.includes(" ") ? raw.replace(" ", "T") + "Z" : raw,
  ];

  for (const candidate of candidates) {
    const parsed = new Date(candidate);
    if (!Number.isNaN(parsed.getTime())) {
      return parsed;
    }
  }

  return null;
}

export function compareTimestampsDesc(left: unknown, right: unknown): number {
  const leftTime = parseTimestamp(left)?.getTime() ?? 0;
  const rightTime = parseTimestamp(right)?.getTime() ?? 0;
  return rightTime - leftTime;
}

export function formatDateTime(value: string | null | undefined, fallback = "-"): string {
  const parsed = parseTimestamp(value);
  if (!parsed) {
    return fallback;
  }

  return parsed.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export function formatDurationBetween(
  start: string | null | undefined,
  end: string | null | undefined
): string {
  const startDate = parseTimestamp(start);
  if (!startDate) {
    return "-";
  }

  const endDate = parseTimestamp(end) || new Date();
  const diffMs = endDate.getTime() - startDate.getTime();
  if (!Number.isFinite(diffMs) || diffMs < 0) {
    return "-";
  }

  const diffSec = Math.floor(diffMs / 1000);
  if (diffSec < 60) {
    return `${diffSec}s`;
  }

  const diffMin = Math.floor(diffSec / 60);
  const remainingSec = diffSec % 60;
  if (diffMin < 60) {
    return `${diffMin}m ${remainingSec}s`;
  }

  const diffHour = Math.floor(diffMin / 60);
  const remainingMin = diffMin % 60;
  return `${diffHour}h ${remainingMin}m`;
}

/**
 * Build-job duration that does the right thing per status:
 *   pending   → "Queued Xs"  (since created_at)
 *   running   → "Running Xs" (since started_at; fallback created_at)
 *   finished  → "Xs"         (started_at → finished_at; fallback to
 *                             created_at → finished_at for jobs that
 *                             pre-date the started_at column)
 */
export function formatJobDuration(job: {
  status: string;
  created_at?: string | null;
  started_at?: string | null;
  finished_at?: string | null;
}): { label: string; value: string } {
  if (job.status === "pending") {
    return {
      label: "Queued",
      value: formatDurationBetween(job.created_at, null),
    };
  }
  if (job.status === "running") {
    return {
      label: "Running",
      value: formatDurationBetween(job.started_at ?? job.created_at, null),
    };
  }
  return {
    label: "Duration",
    value: formatDurationBetween(
      job.started_at ?? job.created_at,
      job.finished_at,
    ),
  };
}

export function formatDurationSeconds(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds <= 0) {
    return "0s";
  }

  const totalSeconds = Math.floor(seconds);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const remainingSeconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  if (minutes > 0) {
    return `${minutes}m ${remainingSeconds}s`;
  }
  return `${remainingSeconds}s`;
}
