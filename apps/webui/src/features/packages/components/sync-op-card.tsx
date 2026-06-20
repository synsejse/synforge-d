import type { SyncOperation } from "../../../lib/types";
import { formatDateTime } from "../../../lib/datetime";
import MetaPair from "../../../components/ui/meta-pair";
import StatusPill from "../../../components/ui/status-pill";

interface SyncOpCardProps {
  op: SyncOperation;
}

const STATUS_RAIL: Record<string, string> = {
  succeeded: "var(--theme-terminal-green)",
  failed: "var(--theme-error-red)",
};

function formatTrigger(trigger: SyncOperation["trigger_type"]): string {
  return trigger.replaceAll("_", " ");
}

export default function SyncOpCard({ op }: SyncOpCardProps) {
  const accent = STATUS_RAIL[op.status] ?? "var(--theme-text-soft)";

  return (
    <article className="relative border border-edge bg-black">
      <span
        aria-hidden="true"
        className="absolute inset-y-0 left-0 w-1"
        style={{ background: accent }}
      />

      <div className="flex flex-col gap-2 pl-4 pr-4 py-3 sm:pl-6 sm:pr-5 sm:py-4">
        <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
          <span className="font-mono text-sm font-bold uppercase tracking-[0.16em] text-white">
            {formatTrigger(op.trigger_type)}
          </span>
          <StatusPill
            status={op.status === "failed" ? "failed" : "succeeded"}
          />
        </div>

        <div className="flex flex-wrap items-start gap-x-6 gap-y-2 font-mono text-xs">
          <MetaPair label="At">
            <span className="text-strong">{formatDateTime(op.created_at)}</span>
          </MetaPair>
          <MetaPair label="Revision">
            <span className="break-all text-strong">
              {op.revision || <em className="not-italic text-soft">—</em>}
            </span>
          </MetaPair>
        </div>

        {op.error_message ? (
          <div className="border-l-2 border-error bg-error/5 px-3 py-2 font-mono text-[11px] text-error">
            {op.error_message}
          </div>
        ) : null}
      </div>
    </article>
  );
}
