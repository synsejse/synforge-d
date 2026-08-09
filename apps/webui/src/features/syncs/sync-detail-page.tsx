import { Suspense, lazy, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { faArrowLeft, faRotate, faStop } from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { syncQueries } from "../../lib/queries";
import { formatDateTime, formatDurationBetween } from "../../lib/datetime";
import ErrorMessage from "../../components/common/error-message";
import { useDialogs } from "../../components/common/dialogs-context";
import { useToast } from "../../components/common/toast-context";
import LoadingBlock from "../../components/ui/loading-block";
import Breadcrumbs from "../../components/ui/breadcrumbs";
import Button from "../../components/ui/button";
import FaIcon from "../../components/ui/fa-icon";
import MetaPair from "../../components/ui/meta-pair";
import StatusPill from "../../components/ui/status-pill";
import Tabs from "../../components/ui/tabs";
import SyncBuildLinks from "./components/sync-build-links";
import SyncTimeline from "./components/sync-timeline";
import CompactId from "../../components/ui/compact-id";
import { formatCompactId } from "../../lib/identifiers";

const TabbedLogViewer = lazy(
  () => import("../jobs/components/tabbed-log-viewer"),
);
type DetailTab = "timeline" | "logs" | "builds";

export default function SyncDetailPage({ operationId }: { operationId: string }) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { confirm } = useDialogs();
  const toast = useToast();
  const [tab, setTab] = useState<DetailTab>("timeline");
  const detailQuery = useQuery({
    ...syncQueries.detail(operationId),
    refetchInterval: (query) => {
      const status = query.state.data?.operation.status;
      return status === "queued" || status === "running" ? 1500 : false;
    },
  });
  const invalidate = () =>
    queryClient.invalidateQueries({ queryKey: ["sync"] });
  const cancelMutation = useMutation({
    mutationFn: () => api.cancelSyncOperation(operationId),
    onSuccess: invalidate,
    onError: (error) =>
      toast.error("Cancel failed", error instanceof Error ? error.message : "Failed to cancel sync"),
  });
  const retryMutation = useMutation({
    mutationFn: () => api.retrySyncOperation(operationId),
    onSuccess: (response) =>
      navigate({ to: "/syncs/view", search: { id: response.operation.id } }),
    onError: (error) =>
      toast.error("Retry failed", error instanceof Error ? error.message : "Failed to retry sync"),
  });

  async function cancel() {
    const ok = await confirm({
      title: "Cancel source sync?",
      message: `Sync ${operationId} will be stopped. Builds already queued are not cancelled.`,
      confirmLabel: "Cancel sync",
      destructive: true,
    });
    if (ok) cancelMutation.mutate();
  }

  if (detailQuery.isPending) {
    return <LoadingBlock label="Loading sync details…" lines={5} />;
  }
  if (detailQuery.error || !detailQuery.data) {
    return <ErrorMessage message={detailQuery.error instanceof Error ? detailQuery.error.message : "Sync not found"} />;
  }

  const { operation, events, builds } = detailQuery.data;
  const live = operation.status === "queued" || operation.status === "running";
  const durationLabel =
    operation.status === "queued"
      ? "Queued for"
      : operation.status === "running"
        ? "Running for"
        : "Duration";
  return (
    <div className="min-w-0 space-y-6">
      <Breadcrumbs
        items={[
          { label: "Syncs", to: "/syncs" },
          { label: operation.package_name, to: "/packages/view", search: { name: operation.package_name } },
          { label: formatCompactId(operation.id) },
        ]}
      />
      <header className="border-b border-edge pb-5">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-3">
              <StatusPill status={operation.status} />
              {operation.changed === false ? (
                <span className="border border-accent-cyan px-2 py-1 font-mono text-xs font-bold uppercase tracking-[0.08em] text-accent-cyan">
                  No changes
                </span>
              ) : null}
              <h1 className="break-all font-mono text-2xl font-bold uppercase text-white sm:text-3xl">
                {operation.package_name}
              </h1>
            </div>
            <div className="mt-4 flex flex-wrap gap-x-8 gap-y-4 font-mono text-xs">
              <MetaPair label="Stage"><span className="text-strong">{operation.stage.replaceAll("_", " ")}</span></MetaPair>
              <MetaPair label="Trigger"><span className="text-strong">{operation.trigger_type.replaceAll("_", " ")}</span></MetaPair>
              <MetaPair label="Target"><span className="text-strong">{operation.target_mock_chroot || "All targets"}</span></MetaPair>
              <MetaPair label="Created"><span className="text-strong">{formatDateTime(operation.created_at)}</span></MetaPair>
              <MetaPair label={durationLabel}><span className={live ? "text-accent-lime" : "text-strong"}>{formatDurationBetween(operation.started_at ?? operation.created_at, live ? undefined : operation.finished_at)}</span></MetaPair>
              <MetaPair label="Revision"><span className="text-strong">{operation.revision || "—"}</span></MetaPair>
              <MetaPair label="Sync"><CompactId value={operation.id} className="text-soft" /></MetaPair>
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            <Button size="sm" variant="ghost" onClick={() => navigate({ to: "/syncs" })}>
              <FaIcon icon={faArrowLeft} /> Back
            </Button>
            {live ? (
              <Button size="sm" variant="warning" loading={cancelMutation.isPending} onClick={() => void cancel()}>
                {cancelMutation.isPending ? null : <FaIcon icon={faStop} />} Cancel
              </Button>
            ) : (
              <Button size="sm" variant="primary" loading={retryMutation.isPending} onClick={() => retryMutation.mutate()}>
                {retryMutation.isPending ? null : <FaIcon icon={faRotate} />} Retry
              </Button>
            )}
          </div>
        </div>
      </header>

      {operation.error_message ? (
        <section className="border border-error bg-error/5 p-5 font-mono text-sm text-error">
          {operation.error_message}
        </section>
      ) : null}

      <div className="grid gap-4 sm:grid-cols-3">
        <TargetCount label="Builds queued" value={operation.queued_targets} tone="text-accent-lime" />
        <TargetCount label="Targets skipped" value={operation.skipped_targets} tone="text-soft" />
        <TargetCount label="Targets blocked" value={operation.blocked_targets} tone="text-accent-orange" />
      </div>

      <Tabs
        value={tab}
        onChange={setTab}
        ariaLabel="Sync detail sections"
        items={[
          { value: "timeline", label: "Timeline", count: events.length },
          { value: "logs", label: "Worker logs" },
          { value: "builds", label: "Builds", count: builds.length },
        ]}
      >
        {tab === "timeline" ? <SyncTimeline events={events} /> : null}
        {tab === "logs" ? (
          <Suspense fallback={<LoadingBlock label="Loading logs…" lines={3} />}>
            <TabbedLogViewer jobId={operation.id} owner="sync" />
          </Suspense>
        ) : null}
        {tab === "builds" ? <SyncBuildLinks builds={builds} /> : null}
      </Tabs>
    </div>
  );
}

function TargetCount({ label, value, tone }: { label: string; value: number; tone: string }) {
  return (
    <div className="border border-edge bg-surface-alt p-4 text-center">
      <div className={`font-mono text-2xl font-bold ${tone}`}>{value}</div>
      <div className="mt-1 font-mono text-xs font-bold uppercase tracking-[0.18em] text-soft">{label}</div>
    </div>
  );
}
