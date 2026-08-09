import type { ReactNode } from "react";
import { cn } from "../../lib/utils";

/**
 * Shared list-row vocabulary used by every record list — repository
 * published files, job artifacts, package build history, sync history.
 * Edit here and all of them update together.
 */

export interface RecordMetaItem {
  label: string;
  value: ReactNode;
}

/** Row container: 1px frame, optional 2px status rail, live pulse, dimming.
 *  The header (title + badges on the left, actions on the right) is built in;
 *  pass the path/meta/footer as children. */
export function RecordCard({
  rail,
  live,
  dimmed,
  title,
  badges,
  actions,
  children,
}: {
  rail?: string;
  live?: boolean;
  dimmed?: boolean;
  title: ReactNode;
  badges?: ReactNode;
  actions?: ReactNode;
  children?: ReactNode;
}) {
  return (
    <article
      className={cn(
        "sf-row relative border border-edge bg-black py-4 pl-[22px] pr-[18px] transition-colors hover:border-edge-strong hover:bg-[#0c0c0d]",
        live && "synforge-row-live",
        dimmed && "opacity-60",
      )}
    >
      {rail ? (
        <span
          aria-hidden="true"
          className="absolute inset-y-0 left-0 w-[2px]"
          style={{ background: rail }}
        />
      ) : null}
      <div className="flex flex-wrap items-center gap-2.5">
        {title}
        {badges}
        {actions ? (
          <div className="ml-auto flex shrink-0 gap-1.5">{actions}</div>
        ) : null}
      </div>
      {children}
    </article>
  );
}

/** Inline LABEL value pairs, wrapping. Pass a coloured node as `value` to
 *  override the default muted tone (e.g. dim ids). */
export function RecordMeta({ items }: { items: RecordMetaItem[] }) {
  return (
    <div className="mt-3 flex flex-wrap gap-x-8 gap-y-2">
      {items.map((item) => (
        <div key={item.label} className="min-w-0">
          <span className="font-mono text-[9px] font-semibold uppercase tracking-[0.16em] text-[#6b6b73]">
            {item.label}{" "}
          </span>
          <span className="break-all font-mono text-[11px] text-muted">
            {item.value}
          </span>
        </div>
      ))}
    </div>
  );
}

/** Neutral bordered metadata chip (package name, target chroot). */
export function RecordChip({ children }: { children: ReactNode }) {
  return (
    <span className="shrink-0 border border-edge bg-black px-[7px] py-1 font-mono text-[9px] font-medium uppercase leading-none tracking-[0.04em] text-[#71717a]">
      {children}
    </span>
  );
}

const KIND_CHIP: Record<string, string> = {
  rpm: "border-success text-success",
  srpm: "border-accent-orange text-accent-orange",
  debuginfo: "border-accent-cyan text-accent-cyan",
  debugsource: "border-accent-cyan text-accent-cyan",
  log: "border-edge-strong text-soft",
  other: "border-edge-strong text-muted",
};

/** Artifact-kind badge (RPM / SRPM / LOG …) with the shared colour map. */
export function KindBadge({ kind }: { kind: string }) {
  return (
    <span
      className={cn(
        "shrink-0 border bg-black px-[7px] py-1 font-mono text-[9px] font-bold uppercase leading-none tracking-[0.08em]",
        KIND_CHIP[kind] ?? "border-edge-strong text-soft",
      )}
    >
      {kind}
    </span>
  );
}

/** Signing state badge (Signed / Sign failed / Not signed). */
export function SigningBadge({
  status,
  errorMessage,
}: {
  status?: string | null;
  errorMessage?: string | null;
}) {
  const state =
    status === "signed"
      ? { label: "Signed", cls: "border-success text-success", dot: "bg-success", title: undefined as string | undefined }
      : status === "failed"
        ? { label: "Sign failed", cls: "border-error text-error", dot: "bg-error", title: errorMessage || "Artifact signing failed" }
        : { label: "Not signed", cls: "border-edge-strong text-soft", dot: "bg-soft", title: undefined as string | undefined };
  return (
    <span
      title={state.title}
      className={cn(
        "inline-flex shrink-0 items-center gap-1.5 border px-[7px] py-1 font-mono text-[9px] font-semibold uppercase leading-none tracking-[0.08em]",
        state.cls,
      )}
    >
      <span aria-hidden="true" className={cn("h-[5px] w-[5px]", state.dot)} />
      {state.label}
    </span>
  );
}
