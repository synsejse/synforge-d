import type { SyncBatch } from "../../../lib/types";
import { Link } from "@tanstack/react-router";
import { formatDateTime, formatDurationBetween } from "../../../lib/datetime";
import StatusPill from "../../../components/ui/status-pill";
import { RecordCard, RecordMeta } from "../../../components/ui/record-card";
import { STATUS_RAIL } from "../../../components/ui/record-card-styles";
import CompactId from "../../../components/ui/compact-id";

export default function SyncBatchCard({ batch }: { batch: SyncBatch }) {
  const live = batch.status === "queued" || batch.status === "running";
  const pct = batch.total_packages
    ? Math.round((batch.completed_packages / batch.total_packages) * 100)
    : 100;
  const durationLabel =
    batch.status === "queued"
      ? "Queued for"
      : batch.status === "running"
        ? "Running for"
        : "Duration";
  return (
    <RecordCard
      rail={STATUS_RAIL[batch.status] ?? "var(--theme-text-soft)"}
      live={live}
      title={
        <Link to="/syncs/batch" search={{ id: batch.id }} className="font-mono text-[15px] font-bold uppercase tracking-[0.02em] text-white transition hover:text-accent-cyan">
          Refresh all
        </Link>
      }
      badges={<StatusPill status={batch.status} />}
    >
      <RecordMeta
        items={[
          { label: "Created", value: formatDateTime(batch.created_at) },
          {
            label: durationLabel,
            value: formatDurationBetween(
              batch.started_at ?? batch.created_at,
              live ? undefined : batch.finished_at,
            ),
          },
          { label: "Progress", value: `${batch.completed_packages} / ${batch.total_packages} (${pct}%)` },
          { label: "Succeeded", value: batch.succeeded_packages },
          { label: "Failed", value: batch.failed_packages },
          { label: "Already active", value: batch.deduplicated_packages },
          {
            label: "Batch",
            value: <CompactId value={batch.id} className="text-soft" />,
          },
        ]}
      />
      <div className="mt-3 h-1.5 overflow-hidden bg-surface-alt">
        <div
          className="h-full bg-accent-cyan transition-[width]"
          style={{ width: `${Math.min(100, pct)}%` }}
        />
      </div>
      {batch.error_message ? (
        <div className="mt-3 border-l-2 border-error bg-error/5 px-3 py-2 font-mono text-xs text-error">
          {batch.error_message}
        </div>
      ) : null}
    </RecordCard>
  );
}
