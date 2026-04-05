export type ByteUnitStyle = "iec" | "metric";

export function formatBytes(
  bytes: number,
  unitStyle: ByteUnitStyle = "iec",
): string {
  if (!Number.isFinite(bytes) || bytes < 0) {
    return "0 B";
  }

  if (bytes < 1024) {
    return `${bytes} B`;
  }

  const units =
    unitStyle === "metric"
      ? ["KB", "MB", "GB", "TB"]
      : ["KiB", "MiB", "GiB", "TiB"];
  let value = bytes / 1024;
  let index = 0;

  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }

  return `${value.toFixed(1)} ${units[index]}`;
}
