import { Suspense, lazy, useEffect, useState } from "react";
import api from "../lib/api";
import DetailStat from "./DetailStat";
import EmptyState from "./EmptyState";
import FaIcon from "./FaIcon";
import LoadingBlock from "./LoadingBlock";
import StatusPill from "./StatusPill";
import { formatDateTime } from "../lib/datetime";
import type { BuildArtifact, BuildJobResponse } from "../lib/types";
import {
  faArrowLeft,
  faCircle,
  faDownload,
  faFolderOpen,
  faRotate,
  faTerminal,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";

interface Props {
  jobId: string;
}

const POLL_INTERVAL_MS = 2000;
const TabbedLogViewer = lazy(() => import("./TabbedLogViewer"));

export default function JobDetail({ jobId }: Props) {
  const [job, setJob] = useState<BuildJobResponse | null>(null);
  const [artifacts, setArtifacts] = useState<BuildArtifact[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [downloadingArtifactPath, setDownloadingArtifactPath] = useState<
    string | null
  >(null);
  const [logsOpen, setLogsOpen] = useState(false);
  const [pageVisible, setPageVisible] = useState(() => {
    if (typeof document === "undefined") {
      return true;
    }
    return document.visibilityState === "visible";
  });

  async function loadJob() {
    try {
      const [jobRes, artifactRes] = await Promise.all([
        api.getJob(jobId),
        api.listJobArtifacts(jobId),
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
    if (typeof document === "undefined") {
      return;
    }
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
    if (!job) {
      return;
    }
    if (job.job.status !== "pending" && job.job.status !== "running") {
      return;
    }
    if (!pageVisible) {
      return;
    }

    loadJob().catch(() => undefined);
    const timer = window.setInterval(() => {
      loadJob().catch(() => undefined);
    }, POLL_INTERVAL_MS);

    return () => window.clearInterval(timer);
  }, [job?.job.status, pageVisible]);

  async function handleDelete() {
    if (!job) {
      return;
    }
    if (
      !confirm(
        `Delete job ${job.job.id}? This only removes stored history and logs.`,
      )
    ) {
      return;
    }
    setDeleting(true);
    try {
      await api.deleteJob(job.job.id);
      window.location.href = "/jobs/";
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to delete job");
      setDeleting(false);
    }
  }

  async function handleArtifactDownload(
    artifact: BuildArtifact,
  ) {
    try {
      setDownloadingArtifactPath(artifact.file);
      await api.downloadJobArtifact(jobId, artifact);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to download artifact");
    } finally {
      setDownloadingArtifactPath(null);
    }
  }

  const isLive = job?.job.status === "pending" || job?.job.status === "running";

  if (loading) {
    return <LoadingBlock label="Loading job…" lines={4} />;
  }

  if (error || !job) {
    return (
      <div className="border border-zinc-800 bg-black p-4 text-zinc-200">
        Error: {error || "Job not found"}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <section className="border border-zinc-800 bg-black p-6">
        <div className="flex flex-col gap-6 xl:flex-row xl:items-end xl:justify-between">
          <div className="space-y-3">
            <a
              href="/jobs/"
              className="text-sm text-zinc-400 transition hover:text-zinc-100"
            >
              <FaIcon icon={faArrowLeft} className="mr-2" />
              Back to jobs
            </a>
            <p className="text-xs uppercase tracking-[0.28em] text-zinc-500">
              Job Trace
            </p>
            <div className="flex flex-wrap items-center gap-3">
              <h1 className="text-4xl font-semibold tracking-tight text-white">
                {job.job.package_name}
              </h1>
              <StatusPill status={job.job.status} />
              {isLive && (
                <span className="inline-flex items-center gap-2 border border-zinc-800 bg-black px-3 py-1 text-xs font-medium uppercase tracking-[0.18em] text-zinc-200">
                  <FaIcon
                    icon={faCircle}
                    className="text-[10px] text-white/80"
                  />
                  Live
                </span>
              )}
            </div>
            <p className="break-all font-mono text-sm text-zinc-400">
              {job.job.revision}
            </p>
          </div>
          <div className="flex flex-wrap gap-3">
            <a
              href={`/packages/view/?name=${encodeURIComponent(job.job.package_name)}`}
              className="border border-zinc-800 bg-black px-4 py-2 text-sm font-medium text-zinc-100 transition hover:border-zinc-600 hover:bg-zinc-950"
            >
              <FaIcon icon={faFolderOpen} className="mr-2" />
              Open Package
            </a>
            <button
              onClick={() => {
                loadJob().catch(() => undefined);
              }}
              className="border border-zinc-800 bg-black px-4 py-2 text-sm font-medium text-zinc-100 transition hover:border-zinc-600 hover:bg-zinc-950"
            >
              <FaIcon icon={faRotate} className="mr-2" />
              Refresh
            </button>
            <button
              onClick={handleDelete}
              disabled={deleting || isLive}
              className="border border-zinc-800 bg-black px-4 py-2 text-sm font-medium text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:cursor-not-allowed disabled:opacity-50"
            >
              <FaIcon icon={faTrash} className="mr-2" />
              {deleting ? "Deleting…" : "Delete Job"}
            </button>
          </div>
        </div>

        <div className="mt-6 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          <DetailStat label="Trigger" value={job.job.trigger} />
          <DetailStat label="Target" value={job.job.mock_chroot} />
          <DetailStat
            label="Created"
            value={formatDateTime(job.job.created_at)}
          />
          <DetailStat
            label="Finished"
            value={formatDateTime(job.job.finished_at, "Still running")}
          />
        </div>
      </section>

      <section className="grid gap-6 xl:grid-cols-[280px_minmax(0,1fr)]">
        <aside className="space-y-4 border border-zinc-800 bg-black p-5">
          <DetailStat label="Job ID" value={job.job.id} mono />
          <DetailStat
            label="Worker Container"
            value={job.job.worker_container_id || "Not assigned"}
            mono
          />
          <DetailStat label="Spec File" value={job.job.spec_file} mono />
          {job.job.error_message && (
            <div className="border border-zinc-800 bg-black p-4">
              <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">
                Error
              </div>
              <pre className="mt-3 whitespace-pre-wrap text-sm text-zinc-200">
                {job.job.error_message}
              </pre>
            </div>
          )}
        </aside>

        <div className="space-y-6">
          {job.job.error_message && (
            <section className="border border-red-700/60 bg-red-500/10 p-5">
              <div>
                <div className="text-xs uppercase tracking-[0.18em] text-red-300">
                  Failure Summary
                </div>
                <pre className="mt-3 whitespace-pre-wrap text-sm text-red-100">
                  {job.job.error_message}
                </pre>
              </div>
            </section>
          )}

          <section className="border border-zinc-800 bg-black p-5 lg:p-6">
            <div className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
              <div>
                <h2 className="text-2xl font-semibold text-white">
                  Build Logs
                </h2>
                <p className="mt-2 text-sm text-zinc-400">
                  Open logs on demand to avoid loading the heavy viewer up
                  front.
                </p>
              </div>
              <button
                type="button"
                onClick={() => setLogsOpen((current) => !current)}
                aria-expanded={logsOpen}
                className="border border-zinc-800 bg-black px-4 py-2 text-sm font-medium text-zinc-100 transition hover:border-zinc-600 hover:bg-zinc-950"
              >
                <FaIcon icon={faTerminal} className="mr-2" />
                {logsOpen ? "Hide Logs" : "Open Logs"}
              </button>
            </div>

            {logsOpen ? (
              <div className="mt-5">
                <Suspense
                  fallback={
                    <LoadingBlock label="Loading log viewer…" lines={5} />
                  }
                >
                  <TabbedLogViewer jobId={jobId} isLive={isLive} />
                </Suspense>
              </div>
            ) : (
              <div className="mt-5 border border-zinc-800 bg-zinc-950/40 p-5 text-sm text-zinc-400">
                The log viewer and its live polling start only after you open
                this section.
              </div>
            )}
          </section>

          <section className="border border-zinc-800 bg-black p-6">
            <div className="mb-5">
              <h2 className="text-xl font-semibold text-white">Artifacts</h2>
              <p className="mt-2 text-sm text-zinc-400">
                Published outputs for this job run.
              </p>
            </div>
            {artifacts.length === 0 ? (
              <EmptyState>No artifacts were recorded for this job.</EmptyState>
            ) : (
              <div className="grid gap-3">
                {artifacts.map((artifact) => (
                  <div
                    key={`${artifact.id}-${artifact.file}`}
                    className="grid gap-3 border border-zinc-800 bg-black px-4 py-4 md:grid-cols-[minmax(0,1fr)_auto_auto_auto]"
                  >
                    <div>
                      <div className="font-mono text-sm text-white">
                        {artifact.file}
                      </div>
                      <div className="mt-1 text-xs text-zinc-500">
                        {artifact.sha256}
                      </div>
                    </div>
                    <div className="text-sm text-zinc-300">
                      {formatBytes(artifact.size_bytes)}
                    </div>
                    <div className="text-sm uppercase tracking-[0.18em] text-zinc-500">
                      {artifact.kind}
                    </div>
                    <div className="flex md:justify-end">
                      <button
                        onClick={() => handleArtifactDownload(artifact)}
                        disabled={downloadingArtifactPath === artifact.file}
                        className="inline-flex items-center border border-zinc-800 bg-black px-3 py-1.5 text-xs text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:cursor-not-allowed disabled:opacity-50"
                      >
                        <FaIcon
                          icon={faDownload}
                          className="mr-2 text-[0.95em]"
                        />
                        {downloadingArtifactPath === artifact.file
                          ? "Downloading…"
                          : "Download"}
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </section>
        </div>
      </section>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
