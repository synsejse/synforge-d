export function formatCompactId(
  value: string,
  leading = 8,
  trailing = 4,
): string {
  const normalized = value.trim();
  if (normalized.length <= leading + trailing + 1) return normalized;
  return `${normalized.slice(0, leading)}…${normalized.slice(-trailing)}`;
}
