import { Suspense, lazy, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import api from "../../../lib/api";
import { jobsQueries } from "../../../lib/queries";
import { formatDateTime, formatJobDuration } from "../../../lib/datetime";
import type {
  JobResourceUsageSample,
  ServerHardwareResponse,
} from "../../../lib/types";
import ErrorMessage from "../../../components/common/error-message";
import PaginationControls from "../../../components/common/pagination-controls";
import { useDialogs } from "../../../components/common/dialogs-provider";
import { useToast } from "../../../components/common/toast-provider";
import { useServerHardware } from "../../../components/common/server-hardware-provider";
import LoadingBlock from "../../../components/ui/loading-block";
import FaIcon from "../../../components/ui/fa-icon";
import Badge from "../../../components/ui/badge";
import Button from "../../../components/ui/button";
import Breadcrumbs from "../../../components/ui/breadcrumbs";
import MetaPair from "../../../components/ui/meta-pair";
import Tabs from "../../../components/ui/tabs";
import ArtifactCard from "./artifact-card";
import {
  faArrowLeft,
  faRotate,
  faStop,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";

interface Props {
  jobId: string;
}

const POLL_INTERVAL_MS = 2000;
const USAGE_POLL_INTERVAL_MS = 1000;
const ARTIFACTS_PAGE_SIZE = 50;
const TabbedLogViewer = lazy(() => import("./tabbed-log-viewer"));

function isLiveStatus(status: string | undefined): boolean {
  return status === "pending" || status === "running";
}

export default function JobDetail({ jobId }: Props) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const { confirm } = useDialogs();
  const toast = useToast();
  const serverHardware = useServerHardware();

  const jobQuery = useQuery({
    ...jobsQueries.detail(jobId),
    refetchInterval: (query) =>
      isLiveStatus(query.state.data?.job.status) ? POLL_INTERVAL_MS : false,
  });

  const isLive = isLiveStatus(jobQuery.data?.job.status);

  const usageQuery = useQuery({
    ...jobsQueries.usage(jobId),
    enabled: isLive,
    refetchInterval: isLive ? USAGE_POLL_INTERVAL_MS : false,
  });
  const latestUsage = usageQuery.data?.sample ?? null;

  const [activeTab, setActiveTab] = useState<"logs" | "artifacts">("logs");
  const [artifactOffset, setArtifactOffset] = useState(0);

  const isDeleted = jobQuery.data?.job.deleted_at != null;

  const artifactsQuery = useQuery({
    ...jobsQueries.artifacts(jobId, {
      limit: ARTIFACTS_PAGE_SIZE,
      offset: artifactOffset,
    }),
    enabled: !isDeleted,
    refetchInterval: isLive ? POLL_INTERVAL_MS : false,
  });
  const artifacts = artifactsQuery.data?.artifacts ?? [];
  const artifactCount =
    artifactsQuery.data?.page.total ?? artifacts.length;

  const invalidateJob = () =>
    queryClient.invalidateQueries({ queryKey: ["jobs"] });

  const deleteMutation = useMutation({
    mutationFn: () => api.deleteJob(jobId),
    onSuccess: () => {
      navigate({ to: "/jobs" });
    },
    onError: (error) =>
      toast.error(
        "Delete failed",
        error instanceof Error ? error.message : "Failed to delete job",
      ),
  });

  const retryMutation = useMutation({
    mutationFn: () => api.retryJob(jobId),
    onSuccess: (response) => {
      navigate({ to: "/jobs/view", search: { id: response.job.id } });
    },
    onError: (error) =>
      toast.error(
        "Retry failed",
        error instanceof Error ? error.message : "Failed to retry job",
      ),
  });

  const killMutation = useMutation({
    mutationFn: () => api.killJob(jobId),
    onSuccess: invalidateJob,
    onError: (error) =>
      toast.error(
        "Kill failed",
        error instanceof Error ? error.message : "Failed to kill job",
      ),
  });

  async function handleDelete() {
    const ok = await confirm({
      title: "Delete job?",
      message: `Job ${jobId} will be removed.`,
      confirmLabel: "Delete",
      destructive: true,
    });
    if (!ok) return;
    deleteMutation.mutate();
  }

  async function handleRetry() {
    if (!jobQuery.data) return;
    const ok = await confirm({
      title: "Retry build?",
      message: `Queue a fresh build for ${jobQuery.data.job.package_name}.`,
      confirmLabel: "Retry",
    });
    if (!ok) return;
    retryMutation.mutate();
  }

  async function handleKill() {
    const ok = await confirm({
      title: "Kill active job?",
      message: `Job ${jobId} will be terminated.`,
      confirmLabel: "Kill",
      destructive: true,
    });
    if (!ok) return;
    killMutation.mutate();
  }

  if (jobQuery.isPending) {
    return (
      <div className="min-w-0 space-y-6">
        <Breadcrumbs
          items={[
            { label: "Jobs", to: "/jobs" },
            { label: jobId },
          ]}
        />
        <LoadingBlock label="Loading job details…" lines={4} />
      </div>
    );
  }

  if (jobQuery.error || !jobQuery.data) {
    return (
      <ErrorMessage
        message={
          jobQuery.error instanceof Error ? jobQuery.error.message : "Job not found"
        }
      />
    );
  }

  const canRetry =
    jobQuery.data.job.status === "succeeded" ||
    jobQuery.data.job.status === "failed" ||
    jobQuery.data.job.status === "timed_out";

  const job = jobQuery.data.job;
  const duration = formatJobDuration(job);

  return (
    <div className="min-w-0 space-y-6">
      <Breadcrumbs
        items={[
          { label: "Jobs", to: "/jobs" },
          {
            label: job.package_name,
            to: "/packages/view",
            search: { name: job.package_name },
          },
          { label: jobId },
        ]}
      />

      <header className="sticky -top-3 z-20 -mx-3 min-w-0 border-b-2 border-edge-strong bg-black/95 px-3 pb-4 pt-3 backdrop-blur-sm sm:-top-5 sm:-mx-5 sm:px-5 lg:-top-8 lg:-mx-8 lg:px-8">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-3">
              <Badge variant={getStatusVariant(job.status)} pulse={isLive}>
                {job.status}
              </Badge>
              {isDeleted ? (
                <span className="border-2 border-edge-strong bg-black px-2 py-1 font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-soft">
                  Deleted
                </span>
              ) : null}
              <h1 className="break-all font-mono text-2xl font-bold uppercase text-white sm:text-3xl">
                {job.package_name}
              </h1>
            </div>
            <div className="mt-3 flex flex-wrap items-start gap-x-6 gap-y-2 font-mono text-xs">
              <MetaPair label="Target">
                <span className="text-strong">{job.mock_chroot}</span>
              </MetaPair>
              <MetaPair label="Trigger">
                <span className="text-strong">{job.trigger}</span>
              </MetaPair>
              <MetaPair label="Created">
                <span className="text-strong">
                  {formatDateTime(job.created_at)}
                </span>
              </MetaPair>
              {job.started_at ? (
                <MetaPair label="Started">
                  <span className="text-strong">
                    {formatDateTime(job.started_at)}
                  </span>
                </MetaPair>
              ) : null}
              {job.finished_at ? (
                <MetaPair label="Finished">
                  <span className="text-strong">
                    {formatDateTime(job.finished_at)}
                  </span>
                </MetaPair>
              ) : null}
              {job.signed_at ? (
                <MetaPair label="Signed">
                  <span className="text-strong">
                    {formatDateTime(job.signed_at)}
                  </span>
                </MetaPair>
              ) : null}
              <MetaPair label={duration.label}>
                <span
                  className={
                    isLive ? "text-accent-lime" : "text-strong"
                  }
                >
                  {duration.value}
                </span>
              </MetaPair>
              <MetaPair label="Revision">
                <span className="break-all text-strong">{job.revision}</span>
              </MetaPair>
              <MetaPair label="Job">
                <span className="break-all text-soft">{job.id}</span>
              </MetaPair>
            </div>
          </div>
          <div className="flex flex-wrap gap-2 lg:flex-nowrap">
            <Button
              variant="ghost"
              size="sm"
              fullWidth="responsive-lg"
              onClick={() => navigate({ to: "/jobs" })}
            >
              <FaIcon icon={faArrowLeft} />
              Back
            </Button>
            {canRetry && !isDeleted && (
              <Button
                variant="primary"
                size="sm"
                fullWidth="responsive-lg"
                onClick={handleRetry}
                loading={retryMutation.isPending}
              >
                {retryMutation.isPending ? null : <FaIcon icon={faRotate} />}
                Retry
              </Button>
            )}
            {isLive && (
              <Button
                variant="warning"
                size="sm"
                fullWidth="responsive-lg"
                onClick={handleKill}
                loading={killMutation.isPending}
              >
                {killMutation.isPending ? null : <FaIcon icon={faStop} />}
                Kill
              </Button>
            )}
            {!isDeleted ? (
              <Button
                variant="danger"
                size="sm"
                fullWidth="responsive-lg"
                onClick={handleDelete}
                loading={deleteMutation.isPending}
                disabled={killMutation.isPending || isLive}
              >
                {deleteMutation.isPending ? null : <FaIcon icon={faTrash} />}
                Delete
              </Button>
            ) : null}
          </div>
        </div>
      </header>

      {isLive ? (
        <LiveUsageStrip
          memoryValue={formatMemoryUsage(latestUsage, serverHardware)}
          memoryPercent={memoryUsagePercent(latestUsage, serverHardware)}
        />
      ) : null}

      {isDeleted ? (
        <section className="border-2 border-edge-strong bg-surface-alt px-4 py-3 sm:px-5">
          <p className="font-mono text-xs text-soft">
            <span className="font-bold uppercase tracking-[0.22em] text-strong">
              Deleted
            </span>
            {job.deleted_at ? (
              <> on {formatDateTime(job.deleted_at)}.</>
            ) : (
              <>.</>
            )}{" "}
            Artifacts and logs are no longer available; this row is kept so
            historical statistics still see the build.
          </p>
        </section>
      ) : null}

      {isDeleted ? null : (
        <Tabs
          ariaLabel="Job detail sections"
          value={activeTab}
          onChange={setActiveTab}
          items={[
            { value: "logs", label: "Build Logs" },
            {
              value: "artifacts",
              label: "Artifacts",
              count: artifactCount > 0 ? artifactCount : null,
              disabled: artifactCount === 0,
            },
          ]}
        >
          {activeTab === "logs" ? (
            <Suspense
              fallback={<LoadingBlock label="Loading logs…" lines={3} />}
            >
              <TabbedLogViewer jobId={jobId} isLive={isLive} />
            </Suspense>
          ) : null}
          {activeTab === "artifacts" ? (
            <div className="space-y-3">
              {artifacts.map((artifact) => (
                <ArtifactCard
                  key={`${artifact.id}:${artifact.file}`}
                  jobId={jobId}
                  artifact={artifact}
                />
              ))}
              {artifactsQuery.data && artifacts.length > 0 ? (
                <PaginationControls
                  offset={artifactOffset}
                  pageSize={ARTIFACTS_PAGE_SIZE}
                  count={artifacts.length}
                  hasMore={artifactsQuery.data.page.has_more}
                  total={artifactsQuery.data.page.total}
                  isFetching={artifactsQuery.isFetching}
                  onOffsetChange={setArtifactOffset}
                />
              ) : null}
            </div>
          ) : null}
        </Tabs>
      )}
    </div>
  );
}

function LiveUsageStrip({
  memoryValue,
  memoryPercent,
}: {
  memoryValue: string;
  memoryPercent: number;
}) {
  const hasSample = memoryValue !== "-";
  return (
    <section
      aria-label="Live resource usage"
      className="flex flex-wrap items-center gap-x-5 gap-y-2 border-2 border-accent-lime bg-black px-4 py-3 sm:px-5"
    >
      <span className="font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-accent-lime">
        Live · Memory
      </span>
      <span className="font-mono text-sm font-bold text-white">
        {memoryValue}
      </span>
      <div className="flex min-w-[140px] flex-1 items-center gap-2">
        <div className="h-2 flex-1 border border-edge-strong bg-black">
          <div
            className="h-full bg-accent-lime transition-all duration-500"
            style={{ width: `${hasSample ? memoryPercent : 0}%` }}
          />
        </div>
        <span className="font-mono text-xs text-soft">
          {hasSample ? `${memoryPercent.toFixed(1)}%` : "—"}
        </span>
      </div>
    </section>
  );
}

function getStatusVariant(status: string) {
  if (status === "succeeded") return "success" as const;
  if (status === "failed" || status === "timed_out") return "error" as const;
  if (status === "running") return "lime" as const;
  if (status === "pending") return "warning" as const;
  return "default" as const;
}

function formatMemory(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GiB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(0)} MiB`;
}

function formatMemoryUsage(
  sample?: JobResourceUsageSample | null,
  hardware?: ServerHardwareResponse | null,
): string {
  if (!sample) return "-";
  const capacityBytes = resolveMemoryCapacityBytes(sample, hardware);
  if (capacityBytes && capacityBytes > 0) {
    return `${formatMemory(sample.memory_usage_bytes)} / ${formatMemory(capacityBytes)}`;
  }
  return formatMemory(sample.memory_usage_bytes);
}

function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.max(0, Math.min(100, value));
}

function resolveMemoryCapacityBytes(
  sample: JobResourceUsageSample,
  hardware?: ServerHardwareResponse | null,
): number | null {
  if (sample.memory_limit_bytes > 0) {
    return sample.memory_limit_bytes;
  }
  const totalMemoryMb = hardware?.total_memory_mb;
  if (totalMemoryMb && totalMemoryMb > 0) {
    return totalMemoryMb * 1024 * 1024;
  }
  return null;
}

function memoryUsagePercent(
  sample?: JobResourceUsageSample | null,
  hardware?: ServerHardwareResponse | null,
): number {
  if (!sample) return 0;
  const capacityBytes = resolveMemoryCapacityBytes(sample, hardware);
  if (!capacityBytes || capacityBytes <= 0) return 0;
  return clampPercent((sample.memory_usage_bytes / capacityBytes) * 100);
}

