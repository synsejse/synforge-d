import { faArrowRight } from "@fortawesome/free-solid-svg-icons";
import { Link } from "@tanstack/react-router";
import type { SyncOperation } from "../../../lib/types";
import { formatDateTime } from "../../../lib/datetime";
import FaIcon from "../../../components/ui/fa-icon";
import StatusPill from "../../../components/ui/status-pill";
import {
  RecordCard,
  RecordChip,
  RecordMeta,
} from "../../../components/ui/record-card";
import { rowActionClass, STATUS_RAIL } from "../../../components/ui/record-card-styles";

export default function SyncRunCard({ operation }: { operation: SyncOperation }) {
  const live = operation.status === "queued" || operation.status === "running";
  return (
    <RecordCard
      rail={STATUS_RAIL[operation.status] ?? "var(--theme-text-soft)"}
      live={live}
      title={
        <Link
          to="/syncs/view"
          search={{ id: operation.id }}
          className="break-all font-mono text-[15px] font-bold uppercase tracking-[0.02em] text-white transition-colors hover:text-accent-cyan"
        >
          {operation.package_name}
        </Link>
      }
      badges={
        <>
          <StatusPill status={operation.status} />
          <RecordChip>{operation.trigger_type.replaceAll("_", " ")}</RecordChip>
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
          { label: "Revision", value: operation.revision || "—" },
          {
            label: "Targets",
            value: `${operation.queued_targets} queued · ${operation.skipped_targets} skipped · ${operation.blocked_targets} blocked`,
          },
          { label: "Sync", value: operation.id },
        ]}
      />
      {operation.error_message ? (
        <div className="mt-3 border-l-2 border-error bg-error/5 px-3 py-2 font-mono text-[11px] text-error">
          {operation.error_message}
        </div>
      ) : null}
    </RecordCard>
  );
}
