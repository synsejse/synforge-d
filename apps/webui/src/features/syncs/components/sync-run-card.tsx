import { faArrowRight } from "@fortawesome/free-solid-svg-icons";
import { Link } from "@tanstack/react-router";
import type { SyncOperation } from "../../../lib/types";
import { formatDateTime, formatDurationBetween } from "../../../lib/datetime";
import FaIcon from "../../../components/ui/fa-icon";
import StatusPill from "../../../components/ui/status-pill";
import {
  RecordCard,
  RecordChip,
  RecordMeta,
} from "../../../components/ui/record-card";
import { rowActionClass, STATUS_RAIL } from "../../../components/ui/record-card-styles";
import CompactId from "../../../components/ui/compact-id";

export default function SyncRunCard({ operation }: { operation: SyncOperation }) {
  const live = operation.status === "queued" || operation.status === "running";
  const durationLabel =
    operation.status === "queued"
      ? "Queued for"
      : operation.status === "running"
        ? "Running for"
        : "Duration";
  return (
    <RecordCard
      rail={STATUS_RAIL[operation.status] ?? "var(--theme-text-soft)"}
      live={live}
      title={
        <Link
          to="/syncs/view"
          search={{ id: operation.id }}
          className="inline-flex min-h-10 items-center break-all font-mono text-[15px] font-bold uppercase tracking-[0.02em] text-white transition-colors hover:text-accent-cyan sm:min-h-0"
        >
          {operation.package_name}
        </Link>
      }
      badges={
        <>
          <StatusPill status={operation.status} />
          <RecordChip>{operation.trigger_type.replaceAll("_", " ")}</RecordChip>
          {operation.changed === false ? (
            <span className="shrink-0 border border-accent-cyan bg-black px-[7px] py-1 font-mono text-xs font-bold uppercase leading-none tracking-[0.06em] text-accent-cyan">
              No changes
            </span>
          ) : null}
          {operation.target_mock_chroot ? (
            <RecordChip>{operation.target_mock_chroot}</RecordChip>
          ) : null}
        </>
      }
      actions={
        <Link
          to="/syncs/view"
          search={{ id: operation.id }}
          className={rowActionClass}
          aria-label={`Open sync ${operation.id}`}
          title="Open sync detail"
        >
          <FaIcon icon={faArrowRight} />
        </Link>
      }
    >
      <RecordMeta
        items={[
          { label: "Stage", value: operation.stage.replaceAll("_", " ") },
          { label: "Created", value: formatDateTime(operation.created_at) },
          {
            label: durationLabel,
            value: formatDurationBetween(
              operation.started_at ?? operation.created_at,
              live ? undefined : operation.finished_at,
            ),
          },
          { label: "Revision", value: operation.revision || "—" },
          {
            label: "Targets",
            value: `${operation.queued_targets} queued · ${operation.skipped_targets} skipped · ${operation.blocked_targets} blocked`,
          },
          {
            label: "Sync",
            value: <CompactId value={operation.id} className="text-soft" />,
          },
        ]}
      />
      {operation.error_message ? (
        <div className="mt-3 border-l-2 border-error bg-error/5 px-3 py-2 font-mono text-xs text-error">
          {operation.error_message}
        </div>
      ) : null}
    </RecordCard>
  );
}
