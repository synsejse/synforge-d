import { Suspense, lazy, useEffect, useState } from "react";
import { jobsApi } from "../api";
import { API_BASE } from "../../../lib/api/client";
import { formatDateTime } from "../../../lib/datetime";
import type {
  BuildArtifact,
  BuildJobResponse,
  JobResourceUsageSample,
  ServerHardwareResponse,
} from "../../../lib/types";
import ErrorMessage from "../../../components/common/ErrorMessage";
import LoadingBlock from "../../../components/ui/LoadingBlock";
import FaIcon from "../../../components/ui/FaIcon";
import Badge from "../../../components/ui/Badge";
import Button from "../../../components/ui/Button";
import {
  faArrowLeft,
  faDownload,
  faRotate,
  faStop,
  faTerminal,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";

interface Props {
  jobId: string;
}

const POLL_INTERVAL_MS = 2000;
const USAGE_POLL_INTERVAL_MS = 1000;
const TabbedLogViewer = lazy(() => import("./TabbedLogViewer"));

export default function JobDetail({ jobId }: Props) {
  const [job, setJob] = useState<BuildJobResponse | null>(null);
  const [artifacts, setArtifacts] = useState<BuildArtifact[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [killing, setKilling] = useState(false);
  const [latestUsage, setLatestUsage] = useState<JobResourceUsageSample | null>(null);
  const [serverHardware, setServerHardware] = useState<ServerHardwareResponse | null>(null);
  const [pageVisible, setPageVisible] = useState(() => {
    if (typeof document === "undefined") return true;
    return document.visibilityState === "visible";
  });

  async function loadJob() {
    try {
      const [jobRes, artifactRes] = await Promise.all([
        jobsApi.getJob(jobId),
        jobsApi.listJobArtifacts(jobId),
      ]);
      setJob(jobRes);
      setArtifacts(artifactRes.artifacts);
      setError(null);
      return jobRes;
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to load job";
      setError(message);
      throw e;
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    loadJob().catch(() => undefined);
  }, [jobId]);

  useEffect(() => {
    api
      .getServerHardware()
      .then((hardware) => setServerHardware(hardware))
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    if (typeof document === "undefined") return;
    const updateVisibility = () => {
      setPageVisible(document.visibilityState === "visible");
    };
    document.addEventListener("visibilitychange", updateVisibility);
    return () => {
      document.removeEventListener("visibilitychange", updateVisibility);
    };
  }, []);

  // Poll for job status updates when live
  useEffect(() => {
    if (!job) return;
    if (job.job.status !== "pending" && job.job.status !== "running") return;
    if (!pageVisible) return;

    loadJob().catch(() => undefined);
    const timer = window.setInterval(() => {
      loadJob().catch(() => undefined);
    }, POLL_INTERVAL_MS);

    return () => window.clearInterval(timer);
  }, [job?.job.status, pageVisible]);

  useEffect(() => {
    if (!job) return;
    if (job.job.status !== "pending" && job.job.status !== "running") return;
    if (!pageVisible) return;
    let cancelled = false;
    const pollUsage = async () => {
      try {
        const response = await jobsApi.getJobUsage(jobId);
        if (cancelled) return;
        setLatestUsage(response.sample ?? null);
      } catch {
        // keep last value when one poll fails
      }
    };
    void pollUsage();
    const timer = window.setInterval(() => {
      void pollUsage();
    }, USAGE_POLL_INTERVAL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [job?.job.status, jobId, pageVisible]);

  async function handleDelete() {
    if (!confirm(`Delete job ${jobId}?`)) return;
    try {
      setDeleting(true);
      await jobsApi.deleteJob(jobId);
      window.location.href = "/jobs/";
    } catch (e) {
      alert(e instanceof Error ? e.message : "Failed to delete job");
      setDeleting(false);
    }
  }

  async function handleRetry() {
    if (!job) return;
    if (!confirm(`Retry build for ${job.job.package_name}?`)) return;
    try {
      const res = await jobsApi.retryJob(jobId);
      window.location.href = `/jobs/view/?id=${encodeURIComponent(res.job.id)}`;
    } catch (e) {
      alert(e instanceof Error ? e.message : "Failed to retry job");
    }
  }

  async function handleKill() {
    if (!confirm(`Kill active job ${jobId}?`)) return;
    try {
      setKilling(true);
      await jobsApi.killJob(jobId);
      await loadJob();
    } catch (e) {
      alert(e instanceof Error ? e.message : "Failed to kill job");
    } finally {
      setKilling(false);
    }
  }

  const getStatusVariant = (status: string) => {
    if (status === "succeeded") return "success";
    if (status === "failed" || status === "timed_out") return "error";
    if (status === "running") return "lime";
    if (status === "pending") return "warning";
    return "default";
  };

  const getArtifactSigningBadge = (artifact: BuildArtifact) => {
    if (artifact.signing_status === "signed") {
      return { label: "SIGNED", variant: "success" as const };
    }
    if (artifact.signing_status === "failed") {
      return {
        label: "SIGN FAILED",
        variant: "error" as const,
        title: artifact.signing_error_message || "Artifact signing failed",
      };
    }
    return { label: "NOT SIGNED", variant: "warning" as const };
  };

  if (loading) {
    return <LoadingBlock label="Loading job details…" lines={6} />;
  }

  if (error || !job) {
    return <ErrorMessage message={error || "Job not found"} />;
  }

  const isLive = job.job.status === "pending" || job.job.status === "running";
  const canRetry =
    job.job.status === "succeeded" ||
    job.job.status === "failed" ||
    job.job.status === "timed_out";

  return (
    <div className="space-y-6">
      {/* Header */}
      <section className="border-4 border-[var(--theme-accent-orange)] bg-black p-6">
        <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-3">
              <Badge variant={getStatusVariant(job.job.status)} pulse={isLive}>
                {job.job.status}
              </Badge>
              <span className="font-mono text-xs font-bold uppercase tracking-[0.3em] text-[var(--theme-accent-orange)]">
                JOB_DETAILS
              </span>
            </div>
            <h1 className="mt-3 font-mono text-3xl font-bold uppercase text-white">
              {job.job.package_name}
            </h1>
            <div className="mt-3 flex flex-wrap items-center gap-4">
              <div className="font-mono text-sm text-zinc-400">
                <span className="text-zinc-600">Target:</span> {job.job.mock_chroot}
              </div>
              <div className="font-mono text-sm text-zinc-400">
                <span className="text-zinc-600">Trigger:</span> {job.job.trigger}
              </div>
              <div className="font-mono text-sm text-zinc-400">
                <span className="text-zinc-600">Created:</span> {formatDateTime(job.job.created_at)}
              </div>
            </div>
          </div>
          <div className="flex flex-wrap gap-3">
            <Button variant="ghost" onClick={() => (window.location.href = "/jobs/")}>
              <FaIcon icon={faArrowLeft} />
              Back
            </Button>
            {canRetry && (
              <Button variant="primary" onClick={handleRetry}>
                <FaIcon icon={faRotate} />
                Retry
              </Button>
            )}
            {isLive && (
              <Button variant="warning" onClick={handleKill} disabled={killing}>
                <FaIcon icon={faStop} />
                Kill Active
              </Button>
            )}
            <Button variant="danger" onClick={handleDelete} disabled={deleting || killing || isLive}>
              <FaIcon icon={faTrash} />
              Delete
            </Button>
          </div>
        </div>
      </section>

      {/* Job Metadata */}
      <div className="border-4 border-[var(--theme-border-strong)] bg-black shadow-[4px_4px_0_rgba(255,255,255,0.1)]">
        <div className="border-b-4 border-[var(--theme-border-strong)] bg-gradient-to-r from-zinc-900 to-black px-6 py-4">
          <h2 className="font-display text-xl font-bold uppercase tracking-tight text-white">
            Build_Metadata
          </h2>
        </div>
        <div className="grid gap-6 p-6 md:grid-cols-2 lg:grid-cols-3">
          <div className="border-l-4 border-[var(--theme-accent-lime)] bg-zinc-950/30 pl-4 pr-3 py-4">
            <div className="font-mono text-xs font-bold uppercase tracking-wider text-zinc-500">
              Job ID
            </div>
            <div className="mt-2 break-all font-mono text-sm text-white">
              {job.job.id}
            </div>
          </div>
          <div className="border-l-4 border-zinc-700 bg-zinc-950/30 pl-4 pr-3 py-4">
            <div className="font-mono text-xs font-bold uppercase tracking-wider text-zinc-500">
              Package
            </div>
            <div className="mt-2 font-display text-base font-bold text-white">
              {job.job.package_name}
            </div>
          </div>
          <div className="border-l-4 border-zinc-700 bg-zinc-950/30 pl-4 pr-3 py-4">
            <div className="font-mono text-xs font-bold uppercase tracking-wider text-zinc-500">
              Mock Chroot
            </div>
            <div className="mt-2 font-mono text-sm text-white">
              {job.job.mock_chroot}
            </div>
          </div>
          <div className="border-l-4 border-zinc-700 bg-zinc-950/30 pl-4 pr-3 py-4">
            <div className="font-mono text-xs font-bold uppercase tracking-wider text-zinc-500">
              Revision
            </div>
            <div className="mt-2 break-all font-mono text-sm text-white">
              {job.job.revision}
            </div>
          </div>
          <div className="border-l-4 border-zinc-700 bg-zinc-950/30 pl-4 pr-3 py-4">
            <div className="font-mono text-xs font-bold uppercase tracking-wider text-zinc-500">
              Created At
            </div>
            <div className="mt-2 font-mono text-sm text-white">
              {formatDateTime(job.job.created_at)}
            </div>
          </div>
          {job.job.finished_at && (
            <div className="border-l-4 border-zinc-700 bg-zinc-950/30 pl-4 pr-3 py-4">
              <div className="font-mono text-xs font-bold uppercase tracking-wider text-zinc-500">
                Finished At
              </div>
              <div className="mt-2 font-mono text-sm text-white">
                {formatDateTime(job.job.finished_at)}
              </div>
            </div>
          )}
        </div>
      </div>

      {isLive && (
        <div className="border-4 border-[var(--theme-accent-lime)] bg-black shadow-[4px_4px_0_rgba(180,255,130,0.2)]">
          <div className="border-b-4 border-[var(--theme-accent-lime)] bg-zinc-950 px-6 py-4">
            <h2 className="font-display text-xl font-bold uppercase tracking-tight text-white">
              Live_Resource_Usage
            </h2>
          </div>
          <div className="p-6">
            <UsageBar
              label="Memory"
              value={formatMemoryUsage(latestUsage, serverHardware)}
              percent={memoryUsagePercent(latestUsage, serverHardware)}
              colorClass="bg-amber-400"
              accentClass="text-amber-300"
            />
          </div>
        </div>
      )}

      {/* Build Logs */}
      <div className="border-4 border-[var(--theme-terminal-green)] bg-black shadow-[4px_4px_0_rgba(0,255,65,0.2)]">
        <div className="border-b-4 border-[var(--theme-terminal-green)] bg-black px-4 py-4 sm:px-6">
          <div className="flex items-center gap-2">
            <FaIcon icon={faTerminal} className="text-[var(--theme-terminal-green)]" />
            <h2 className="font-display text-xl font-bold uppercase tracking-tight text-[var(--theme-terminal-green)]">
              Build_Logs
            </h2>
          </div>
        </div>
        <div className="bg-black p-0">
          <Suspense fallback={<div className="p-6"><LoadingBlock label="Loading logs…" lines={3} /></div>}>
            <TabbedLogViewer jobId={jobId} isLive={isLive} />
          </Suspense>
        </div>
      </div>

      {/* Artifacts */}
      {artifacts.length > 0 && (
        <div className="border-4 border-[var(--theme-border-strong)] bg-black shadow-[4px_4px_0_rgba(255,255,255,0.1)]">
          <div className="border-b-4 border-[var(--theme-border-strong)] bg-gradient-to-r from-zinc-900 to-black px-6 py-4">
            <h2 className="font-display text-xl font-bold uppercase tracking-tight text-white">
              Build_Artifacts
            </h2>
          </div>
          <div className="p-6">
            <div className="grid gap-2">
              {artifacts.map((artifact) => {
                const signingBadge = getArtifactSigningBadge(artifact);
                return (
                  <div
                    key={`${artifact.id}:${artifact.file}`}
                    className="flex flex-col gap-4 border-2 border-[var(--theme-border)] bg-zinc-950/40 px-4 py-4 transition-all hover:border-[var(--theme-border-strong)] hover:bg-zinc-950 sm:flex-row sm:items-center sm:justify-between sm:px-5"
                  >
                    <div className="min-w-0 flex-1">
                      <div className="break-all font-mono text-sm text-white">
                        {artifact.file}
                      </div>
                      <div className="mt-1 font-mono text-xs text-zinc-600">
                        {artifact.size_bytes.toLocaleString()} bytes
                      </div>
                    </div>
                    <div className="grid gap-2 sm:flex sm:items-center sm:gap-3">
                      <Badge variant={signingBadge.variant} title={signingBadge.title}>
                        {signingBadge.label}
                      </Badge>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="w-full sm:w-auto"
                        onClick={() => {
                          const encodedFile = artifact.file
                            .split("/")
                            .map((segment) => encodeURIComponent(segment))
                            .join("/");
                          const url = `${API_BASE}/api/v1/jobs/${encodeURIComponent(jobId)}/artifacts/${encodedFile}/content`;
                          window.open(url, "_blank");
                        }}
                      >
                        <FaIcon icon={faDownload} />
                        Download
                      </Button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      )}
    </div>
  );
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
  if (hardware && hardware.total_memory_mb > 0) {
    return hardware.total_memory_mb * 1024 * 1024;
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

function UsageBar({
  label,
  value,
  percent,
  colorClass,
  accentClass,
}: {
  label: string;
  value: string;
  percent: number;
  colorClass: string;
  accentClass: string;
}) {
  const hasSample = value !== "-";
  return (
    <div className="space-y-3 border-2 border-zinc-700 bg-zinc-950 p-5">
      <div className="flex items-center justify-between gap-3">
        <span className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-500">
          {label}
        </span>
        <span className={`font-mono text-sm font-bold uppercase tracking-[0.12em] ${accentClass}`}>
          {value}
        </span>
      </div>
      <div className="h-10 border-2 border-zinc-700 bg-black p-1">
        <div
          className={`h-full ${colorClass} transition-all duration-700`}
          style={{ width: `${hasSample ? percent : 0}%` }}
        />
      </div>
      <div className="font-mono text-xs uppercase tracking-[0.1em] text-zinc-500">
        {hasSample ? `${percent.toFixed(1)}%` : "WAITING_FOR_SAMPLE"}
      </div>
    </div>
  );
}
