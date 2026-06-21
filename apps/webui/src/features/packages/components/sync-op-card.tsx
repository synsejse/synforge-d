import type { SyncOperation } from "../../../lib/types";
import { formatDateTime } from "../../../lib/datetime";
import StatusPill from "../../../components/ui/status-pill";
import {
  RecordCard,
  RecordMeta,
  STATUS_RAIL,
} from "../../../components/ui/record-card";

interface SyncOpCardProps {
  op: SyncOperation;
}

function formatTrigger(trigger: SyncOperation["trigger_type"]): string {
  return trigger.replaceAll("_", " ");
}

export default function SyncOpCard({ op }: SyncOpCardProps) {
  const rail = STATUS_RAIL[op.status] ?? "var(--theme-text-soft)";

  return (
    <RecordCard
      rail={rail}
      title={
        <span className="font-mono text-[15px] font-bold uppercase tracking-[0.02em] text-white">
          {formatTrigger(op.trigger_type)}
        </span>
      }
      badges={<StatusPill status={op.status === "failed" ? "failed" : "succeeded"} />}
    >
      <RecordMeta
        items={[
          { label: "At", value: formatDateTime(op.created_at) },
          { label: "Revision", value: op.revision || "—" },
        ]}
      />
      {op.error_message ? (
        <div className="mt-3 border-l-2 border-error bg-error/5 px-3 py-2 font-mono text-[11px] text-error">
          {op.error_message}
        </div>
      ) : null}
    </RecordCard>
  );
}
