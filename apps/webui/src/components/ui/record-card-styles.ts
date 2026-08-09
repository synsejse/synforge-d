export const rowActionClass =
  "sf-ic inline-flex h-10 w-10 shrink-0 items-center justify-center border border-edge bg-transparent text-soft transition-colors hover:border-accent-lime hover:text-accent-lime focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime sm:h-[30px] sm:w-[30px]";

export const ACCENT_RAIL = "var(--theme-accent-lime)";
export const ERROR_RAIL = "var(--theme-error-red)";
export const STATUS_RAIL: Record<string, string> = {
  succeeded: "var(--theme-terminal-green)",
  failed: "var(--theme-error-red)",
  timed_out: "var(--theme-error-red)",
  running: "var(--theme-accent-lime)",
  pending: "var(--theme-accent-orange)",
  queued: "var(--theme-accent-orange)",
  cancelled: "var(--theme-text-soft)",
  interrupted: "var(--theme-error-red)",
};
