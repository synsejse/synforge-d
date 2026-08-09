import ProgressOverlayDialog from "../../../components/ui/progress-overlay-dialog";
import type { RefreshAllPackagesProgressView } from "../../../lib/types";

function refreshTitle(state: RefreshAllPackagesProgressView["state"]): string {
  if (state === "running") return "Refreshing enabled packages";
  if (state === "failed") return "Refresh all failed";
  return "Refresh all complete";
}

function refreshTone(
  state: RefreshAllPackagesProgressView["state"],
): "running" | "success" | "error" {
  if (state === "failed") return "error";
  if (state === "running") return "running";
  return "success";
}

function StatRow({
  label,
  value,
  emphasis,
}: {
  label: string;
  value: number;
  emphasis?: "lime" | "error";
}) {
  const valueClass =
    value === 0
      ? "text-soft"
      : emphasis === "error"
        ? "text-error"
        : emphasis === "lime"
          ? "text-accent-lime"
          : "text-white";
  return (
    <div className="flex items-baseline justify-between gap-3">
      <span className="font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-soft">
        {label}
      </span>
      <span className={`font-mono text-sm font-bold ${valueClass}`}>{value}</span>
    </div>
  );
}

function RefreshAllStats({
  operation,
}: {
  operation: RefreshAllPackagesProgressView;
}) {
  return (
    <div className="grid grid-cols-2 gap-px border border-edge bg-edge-strong">
      <div className="space-y-2 bg-black px-4 py-3">
        <div className="font-mono text-[10px] font-bold uppercase tracking-[0.24em] text-soft">
          Packages
        </div>
        <StatRow label="Queued" value={operation.queued_packages} emphasis="lime" />
        <StatRow label="Skipped" value={operation.skipped_packages} />
        <StatRow label="Blocked" value={operation.blocked_packages} />
        <StatRow
          label="Failed"
          value={operation.failed_packages}
          emphasis="error"
        />
      </div>
      <div className="space-y-2 bg-black px-4 py-3">
        <div className="font-mono text-[10px] font-bold uppercase tracking-[0.24em] text-soft">
          Targets
        </div>
        <StatRow label="Queued" value={operation.queued_targets} emphasis="lime" />
        <StatRow label="Skipped" value={operation.skipped_targets} />
        <StatRow label="Blocked" value={operation.blocked_targets} />
      </div>
    </div>
  );
}

interface Props {
  open: boolean;
  operation: RefreshAllPackagesProgressView | null;
  closeDisabled: boolean;
  onClose: () => void;
}

export default function RefreshAllProgressDialog({
  open,
  operation,
  closeDisabled,
  onClose,
}: Props) {
  const title = operation
    ? refreshTitle(operation.state)
    : "Refreshing enabled packages";
  const tone = operation ? refreshTone(operation.state) : "running";
  const progress = operation
    ? operation.total_packages === 0
      ? operation.state === "running"
        ? 0
        : 100
      : Math.min(
          100,
          Math.round(
            (operation.processed_packages / operation.total_packages) * 100,
          ),
        )
    : 0;
  const summary = operation
    ? operation.total_packages === 0
      ? "Preparing…"
      : `${operation.processed_packages} / ${operation.total_packages} packages`
    : "Preparing…";
  const message =
    operation?.message && operation.state !== "completed"
      ? operation.message
      : null;

  return (
    <ProgressOverlayDialog
      open={open}
      title={title}
      tone={tone}
      summary={summary}
      progress={progress}
      onClose={onClose}
      closeDisabled={closeDisabled}
    >
      {operation && operation.total_packages > 0 ? (
        <RefreshAllStats operation={operation} />
      ) : null}
      {message ? (
        <p
          className={`mt-4 font-mono text-xs ${
            tone === "error" ? "text-error" : "text-soft"
          }`}
        >
          {message}
        </p>
      ) : null}
    </ProgressOverlayDialog>
  );
}
