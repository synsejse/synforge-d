import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { syncQueries } from "../../lib/queries";
import { formatDateTime } from "../../lib/datetime";
import ErrorMessage from "../../components/common/error-message";
import LoadingBlock from "../../components/ui/loading-block";
import Breadcrumbs from "../../components/ui/breadcrumbs";
import Button from "../../components/ui/button";
import StatusPill from "../../components/ui/status-pill";
import SyncRunCard from "./components/sync-run-card";

export default function SyncBatchDetailPage({ batchId }: { batchId: string }) {
  const navigate = useNavigate();
  const query = useQuery({
    ...syncQueries.batch(batchId),
    refetchInterval: (result) => {
      const status = result.state.data?.batch.status;
      return status === "queued" || status === "running" ? 1500 : false;
    },
  });
  if (query.isPending) return <LoadingBlock label="Loading refresh batch…" lines={5} />;
  if (query.error || !query.data) {
    return <ErrorMessage message={query.error instanceof Error ? query.error.message : "Batch not found"} />;
  }
  const { batch, operations } = query.data;
  const pct = batch.total_packages
    ? Math.round((batch.completed_packages / batch.total_packages) * 100)
    : 100;
  return (
    <div className="space-y-6">
      <Breadcrumbs items={[{ label: "Syncs", to: "/syncs" }, { label: batch.id }]} />
      <header className="border-b border-edge pb-5">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <div className="flex items-center gap-3">
              <StatusPill status={batch.status} />
              <h1 className="font-mono text-2xl font-bold uppercase text-white sm:text-3xl">Refresh all</h1>
            </div>
            <p className="mt-2 font-mono text-xs text-soft">
              {formatDateTime(batch.created_at)} · {batch.id}
            </p>
          </div>
          <Button size="sm" variant="ghost" onClick={() => navigate({ to: "/syncs", search: { mode: "batches" } })}>
            Back to batches
          </Button>
        </div>
      </header>
      <section className="border border-edge bg-surface-alt p-5 sm:p-6">
        <div className="flex flex-wrap items-baseline justify-between gap-3">
          <span className="font-mono text-xs font-bold uppercase tracking-[0.18em] text-soft">Batch progress</span>
          <span className="font-mono text-xl font-bold text-accent-cyan">{batch.completed_packages} / {batch.total_packages} · {pct}%</span>
        </div>
        <div className="mt-4 h-2 bg-black"><div className="h-full bg-accent-cyan" style={{ width: `${Math.min(100, pct)}%` }} /></div>
        <div className="mt-5 grid grid-cols-2 gap-4 sm:grid-cols-5">
          <BatchCount label="Succeeded" value={batch.succeeded_packages} />
          <BatchCount label="Failed" value={batch.failed_packages} />
          <BatchCount label="Cancelled" value={batch.cancelled_packages} />
          <BatchCount label="Already active" value={batch.deduplicated_packages} />
          <BatchCount label="Enqueue errors" value={batch.enqueue_failed_packages} />
        </div>
      </section>
      <section className="space-y-4">
        <h2 className="font-mono text-sm font-bold uppercase tracking-[0.16em] text-white">Package runs</h2>
        {operations.length > 0 ? (
          <div className="space-y-3">{operations.map((operation) => <SyncRunCard key={operation.id} operation={operation} />)}</div>
        ) : (
          <p className="font-mono text-sm text-soft">No new runs were required for this batch.</p>
        )}
      </section>
    </div>
  );
}

function BatchCount({ label, value }: { label: string; value: number }) {
  return <div><div className="font-mono text-xl font-bold text-white">{value}</div><div className="mt-1 font-mono text-xs uppercase tracking-[0.14em] text-soft">{label}</div></div>;
}
