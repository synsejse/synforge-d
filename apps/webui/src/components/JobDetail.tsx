import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import api from "../lib/api";
import DetailStat from "./DetailStat";
import EmptyState from "./EmptyState";
import FaIcon from "./FaIcon";
import StatusPill from "./StatusPill";
import { formatDateTime } from "../lib/datetime";
import type { BuildJobResponse } from "../lib/types";
import {
  faArrowLeft,
  faCircle,
  faDownload,
  faFolderOpen,
  faRotate,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";

interface Props {
  jobId: string;
}

const POLL_INTERVAL_MS = 2000;

export default function JobDetail({ jobId }: Props) {
  const [job, setJob] = useState<BuildJobResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [logText, setLogText] = useState("");
  const [cursor, setCursor] = useState(0);
  const [logLoading, setLogLoading] = useState(true);
  const [followLogs, setFollowLogs] = useState(true);
  const [deleting, setDeleting] = useState(false);
  const [downloadingArtifactPath, setDownloadingArtifactPath] = useState<string | null>(null);
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({});
  const logViewportRef = useRef<HTMLDivElement | null>(null);

  async function loadJob() {
    try {
      const jobRes = await api.getJob(jobId);
      setJob(jobRes);
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

  async function loadLogChunk(reset = false) {
    try {
      const nextCursor = reset ? 0 : cursor;
      const res = await api.getJobLogChunk(jobId, nextCursor);
      setLogText((current) => (reset ? res.contents : current + res.contents));
      setCursor(res.cursor);
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to load job log";
      setError(message);
    } finally {
      setLogLoading(false);
    }
  }

  useEffect(() => {
    let cancelled = false;

    async function initialLoad() {
      const jobRes = await loadJob();
      if (!cancelled) {
        await loadLogChunk(true);
        if (jobRes.job.status === "pending" || jobRes.job.status === "running") {
          setFollowLogs(true);
        }
      }
    }

    initialLoad().catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [jobId]);

  useEffect(() => {
    if (!job || !followLogs) {
      return;
    }
    if (job.job.status !== "pending" && job.job.status !== "running") {
      return;
    }

    const timer = window.setInterval(async () => {
      await loadJob();
      await loadLogChunk();
    }, POLL_INTERVAL_MS);

    return () => window.clearInterval(timer);
  }, [job, followLogs, cursor]);

  useEffect(() => {
    const node = logViewportRef.current;
    if (!node || !followLogs) {
      return;
    }
    node.scrollTop = node.scrollHeight;
  }, [logText, followLogs]);

  async function handleDelete() {
    if (!job) {
      return;
    }
    if (!confirm(`Delete job ${job.job.id}? This only removes stored history and logs.`)) {
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

  async function handleArtifactDownload(artifact: BuildJobResponse["artifacts"][number]) {
    try {
      setDownloadingArtifactPath(artifact.relative_repo_path);
      await api.downloadJobArtifact(jobId, artifact);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to download artifact");
    } finally {
      setDownloadingArtifactPath(null);
    }
  }

  const logLines = useMemo(() => {
    const lines = logText.split("\n");
    if (lines.at(-1) === "") {
      lines.pop();
    }
    return lines;
  }, [logText]);
  const logSections = useMemo(() => parseLogSections(logLines), [logLines]);
  const hasRenderableSections = logSections.some((section) => section.lines.length > 0);
  const isLive = job?.job.status === "pending" || job?.job.status === "running";
  const latestSectionKey = logSections.at(-1)?.key ?? null;
  const failureSection = useMemo(() => {
    const status = job?.job.status;
    if (status !== "failed" && status !== "timed_out") {
      return null;
    }
    for (let index = logSections.length - 1; index >= 0; index -= 1) {
      const section = logSections[index];
      if (section.lines.some((line) => line.trim().length > 0)) {
        return section;
      }
    }
    return logSections.at(-1) ?? null;
  }, [job?.job.status, logSections]);

  useEffect(() => {
    if (!failureSection) {
      return;
    }
    setExpandedSections((current) => {
      if (current[failureSection.key]) {
        return current;
      }
      return { ...current, [failureSection.key]: true };
    });
  }, [failureSection]);

  if (loading) {
    return <div className="text-zinc-400">Loading job…</div>;
  }

  if (error || !job) {
    return <div className="border border-zinc-800 bg-black p-4 text-zinc-200">Error: {error || "Job not found"}</div>;
  }

  return (
    <div className="space-y-6">
      <section className="border border-zinc-800 bg-black p-6">
        <div className="flex flex-col gap-6 xl:flex-row xl:items-end xl:justify-between">
          <div className="space-y-3">
            <a href="/jobs/" className="text-sm text-zinc-400 transition hover:text-zinc-100">
              <FaIcon icon={faArrowLeft} className="mr-2" />
              Back to jobs
            </a>
            <p className="text-xs uppercase tracking-[0.28em] text-zinc-500">Job Trace</p>
            <div className="flex flex-wrap items-center gap-3">
              <h1 className="text-4xl font-semibold tracking-tight text-white">{job.job.package_name}</h1>
              <StatusPill status={job.job.status} />
              {isLive && (
                <span className="inline-flex items-center gap-2 border border-zinc-800 bg-black px-3 py-1 text-xs font-medium uppercase tracking-[0.18em] text-zinc-200">
                  <FaIcon icon={faCircle} className="text-[10px] text-white/80" />
                  Live
                </span>
              )}
            </div>
            <p className="break-all font-mono text-sm text-zinc-400">{job.job.revision}</p>
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
                setCursor(0);
                setLogText("");
                setLogLoading(true);
                loadJob().then(() => loadLogChunk(true)).catch(() => undefined);
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
          <DetailStat label="Created" value={formatDateTime(job.job.created_at)} />
          <DetailStat label="Finished" value={formatDateTime(job.job.finished_at, "Still running")} />
        </div>
      </section>

      <section className="grid gap-6 xl:grid-cols-[280px_minmax(0,1fr)]">
        <aside className="space-y-4 border border-zinc-800 bg-black p-5">
          <DetailStat label="Job ID" value={job.job.id} mono />
          <DetailStat label="Worker Container" value={job.job.worker_container_id || "Not assigned"} mono />
          <DetailStat label="Spec Path" value={job.job.spec_path} mono />
          {job.job.error_message && (
            <div className="border border-zinc-800 bg-black p-4">
              <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">Error</div>
              <pre className="mt-3 whitespace-pre-wrap text-sm text-zinc-200">{job.job.error_message}</pre>
            </div>
          )}
        </aside>

        <div className="space-y-6">
          {job.job.error_message && (
            <section className="border border-red-700/60 bg-red-500/10 p-5">
              <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                <div>
                  <div className="text-xs uppercase tracking-[0.18em] text-red-300">Failure Summary</div>
                  <pre className="mt-3 whitespace-pre-wrap text-sm text-red-100">{job.job.error_message}</pre>
                </div>
                {failureSection && (
                  <button
                    type="button"
                    onClick={() => {
                      setExpandedSections((current) => ({ ...current, [failureSection.key]: true }));
                      requestAnimationFrame(() => {
                        logViewportRef.current?.scrollTo({ top: 0, behavior: "smooth" });
                      });
                    }}
                    className="border border-red-700/60 bg-red-500/10 px-4 py-2 text-sm font-medium text-red-100 transition hover:bg-red-500/20"
                  >
                    Open failing section
                  </button>
                )}
              </div>
            </section>
          )}

          <section className="border border-zinc-800 bg-black p-5 lg:p-6">
            <div className="mb-5 flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
              <div>
                <h2 className="text-2xl font-semibold text-white">Live Logs</h2>
                <p className="mt-2 text-sm text-zinc-400">
                  Incremental log streaming with follow mode, suited for watching active builds.
                </p>
              </div>
              <label className="inline-flex items-center gap-3 border border-zinc-800 bg-black px-4 py-2 text-sm text-zinc-300">
                <input
                  type="checkbox"
                  checked={followLogs}
                  onChange={(event) => setFollowLogs(event.target.checked)}
                  className="h-4 w-4 rounded border-zinc-700 bg-zinc-900"
                />
                Follow output
              </label>
            </div>

            <div className="mb-4 flex flex-wrap items-center gap-2 text-[11px] uppercase tracking-[0.18em] text-zinc-500">
              <MetricPill>{logLines.length} lines</MetricPill>
              <MetricPill>{logSections.length} sections</MetricPill>
              <MetricPill>{cursor} bytes</MetricPill>
              <MetricPill>{logLoading ? "Fetching" : isLive ? "Polling active" : "Final log"}</MetricPill>
              <MetricPill>Incremental stream</MetricPill>
            </div>

            <div
              ref={logViewportRef}
              className="max-h-[78vh] overflow-auto border border-zinc-800 bg-black"
            >
              {logLines.length === 0 && !logLoading ? (
                <EmptyState>No log content available yet.</EmptyState>
              ) : !hasRenderableSections ? (
                <pre className="whitespace-pre-wrap break-words p-5 font-mono text-[14px] leading-6 text-slate-100">
                  {logText || "Waiting for output…"}
                </pre>
              ) : (
                <div className="space-y-4 p-4">
                  {logSections.map((section) => (
                    <LogSection
                      key={section.key}
                      section={section}
                      isExpanded={Boolean(expandedSections[section.key])}
                      isLatest={section.key === latestSectionKey}
                      isFailureSection={section.key === failureSection?.key}
                      viewportRef={logViewportRef}
                      onToggle={(isOpen) => {
                        setExpandedSections((current) => ({ ...current, [section.key]: isOpen }));
                      }}
                    />
                  ))}
                </div>
              )}
            </div>
          </section>

          <section className="border border-zinc-800 bg-black p-6">
            <div className="mb-5">
              <h2 className="text-xl font-semibold text-white">Artifacts</h2>
              <p className="mt-2 text-sm text-zinc-400">Published outputs for this job run.</p>
            </div>
            {job.artifacts.length === 0 ? (
              <EmptyState>No artifacts were recorded for this job.</EmptyState>
            ) : (
              <div className="grid gap-3">
                {job.artifacts.map((artifact) => (
                  <div
                    key={`${artifact.path}-${artifact.relative_repo_path}`}
                    className="grid gap-3 border border-zinc-800 bg-black px-4 py-4 md:grid-cols-[minmax(0,1fr)_auto_auto_auto]"
                  >
                    <div>
                      <div className="font-mono text-sm text-white">{artifact.relative_repo_path}</div>
                      <div className="mt-1 text-xs text-zinc-500">{artifact.sha256}</div>
                    </div>
                    <div className="text-sm text-zinc-300">{formatBytes(artifact.size_bytes)}</div>
                    <div className="text-sm uppercase tracking-[0.18em] text-zinc-500">{artifact.kind}</div>
                    <div className="flex md:justify-end">
                      <button
                        onClick={() => handleArtifactDownload(artifact)}
                        disabled={downloadingArtifactPath === artifact.relative_repo_path}
                        className="inline-flex items-center border border-zinc-800 bg-black px-3 py-1.5 text-xs text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:cursor-not-allowed disabled:opacity-50"
                      >
                        <FaIcon icon={faDownload} className="mr-2 text-[0.95em]" />
                        {downloadingArtifactPath === artifact.relative_repo_path ? "Downloading…" : "Download"}
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

function MetricPill({ children }: { children: ReactNode }) {
  return (
    <span className="border border-zinc-800 bg-black px-3 py-1.5">
      {children}
    </span>
  );
}

function LogSection({
  section,
  isExpanded,
  isLatest,
  isFailureSection,
  viewportRef,
  onToggle,
}: {
  section: ReturnType<typeof parseLogSections>[number];
  isExpanded: boolean;
  isLatest: boolean;
  isFailureSection: boolean;
  viewportRef: React.RefObject<HTMLDivElement | null>;
  onToggle: (isOpen: boolean) => void;
}) {
  const sectionBodyRef = useRef<HTMLDivElement | null>(null);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    if (!isExpanded) {
      setIsVisible(false);
      return;
    }
    const node = sectionBodyRef.current;
    const root = viewportRef.current;
    if (!node || !root) {
      setIsVisible(true);
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        const [entry] = entries;
        setIsVisible(Boolean(entry?.isIntersecting));
      },
      {
        root,
        rootMargin: "240px 0px",
      }
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [isExpanded, viewportRef, section.key]);

  return (
    <details
      className="overflow-hidden border border-zinc-800 bg-black"
      open={isExpanded}
      onToggle={(event) => {
        onToggle((event.currentTarget as HTMLDetailsElement).open);
      }}
    >
      <summary className="cursor-pointer list-none border-b border-zinc-800 bg-zinc-950 px-4 py-3 marker:hidden">
        <div className="flex flex-col gap-2 md:flex-row md:items-start md:justify-between">
          <div className="min-w-0">
            <div className="text-xs uppercase tracking-[0.18em] text-zinc-500">Section</div>
            <div className="mt-1 text-sm font-medium text-zinc-100">{section.title}</div>
            {!isExpanded && isLatest && (
              <div className="mt-2 line-clamp-1 font-mono text-xs text-zinc-400">
                {section.preview || "Waiting for output…"}
              </div>
            )}
          </div>
          <div className="flex flex-wrap items-center gap-2 text-[11px] uppercase tracking-[0.18em] text-zinc-500">
            <MetricPill>{section.lines.length} lines</MetricPill>
            <MetricPill>{section.startLine}-{section.endLine}</MetricPill>
            {isLatest && !isExpanded && <MetricPill>latest</MetricPill>}
            {isFailureSection && <MetricPill>failure</MetricPill>}
          </div>
        </div>
      </summary>
      {isExpanded && (
        <div ref={sectionBodyRef}>
          {!isVisible ? (
            <div className="px-4 py-4 text-sm text-zinc-500">Scroll this section into view to render its lines.</div>
          ) : (
            <div className="min-w-full divide-y divide-white/5 font-mono text-[14px] leading-6">
              {section.lines.length === 0 ? (
                <div className="px-4 py-3 text-zinc-500">Waiting for output…</div>
              ) : (
                section.lines.map((line, lineIndex) => (
                  <div
                    key={`${section.key}-${lineIndex}-${line.slice(0, 12)}`}
                    className="grid grid-cols-[64px_minmax(0,1fr)] items-start"
                  >
                    <div className="select-none border-r border-zinc-800 bg-zinc-950 px-4 py-1 text-right text-zinc-500">
                      {section.startLine + lineIndex}
                    </div>
                    <pre className="whitespace-pre-wrap break-words px-4 py-1 text-slate-100">{line || " "}</pre>
                  </div>
                ))
              )}
            </div>
          )}
        </div>
      )}
    </details>
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

function parseLogSections(lines: string[]) {
  const sections: Array<{
    key: string;
    title: string;
    lines: string[];
    startLine: number;
    endLine: number;
    preview: string;
  }> = [];
  let current = { title: "Job output", lines: [] as string[], startLine: 1 };
  let visibleLine = 1;

  for (const line of lines) {
    if (line.startsWith("::section::")) {
      if (current.lines.length > 0 || sections.length === 0) {
        sections.push(toRenderableSection(current, sections.length));
      }
      current = {
        title: line.slice("::section::".length).trim() || "Untitled section",
        lines: [],
        startLine: visibleLine,
      };
      continue;
    }
    current.lines.push(line);
    visibleLine += 1;
  }

  if (current.lines.length > 0 || sections.length === 0) {
    sections.push(toRenderableSection(current, sections.length));
  }

  return sections.filter((section, index) => section.lines.length > 0 || index === sections.length - 1);
}

function toRenderableSection(
  section: { title: string; lines: string[]; startLine: number },
  index: number
) {
  const preview = [...section.lines].reverse().find((line) => line.trim().length > 0) || "";
  return {
    key: `${index}-${section.title}`,
    title: section.title,
    lines: section.lines,
    startLine: section.startLine,
    endLine: Math.max(section.startLine, section.startLine + section.lines.length - 1),
    preview,
  };
}
