import { useEffect, useMemo, useState } from "react";
import {
  faDownload,
  faMagnifyingGlass,
  faXmark,
} from "@fortawesome/free-solid-svg-icons";
import { API_BASE } from "../../../lib/api/client";
import { downloadLogText } from "../../../lib/api/jobs";
import EmptyState from "../../../components/ui/empty-state";
import FaIcon from "../../../components/ui/fa-icon";
import { useDebounce } from "../../../lib/hooks/use-debounce";
import { GroupedLogView, VirtualizedAnsiLines } from "./log-lines-view";

interface Props {
  jobId: string;
  owner?: "job" | "sync";
}

type StreamStatus = "connecting" | "live" | "reconnecting" | "complete";

const PRIMARY_SOURCE = "worker.log";
const MOBILE_LOG_VIEWPORT_HEIGHT = 420;
const DESKTOP_LOG_VIEWPORT_HEIGHT = 600;

/**
 * Subscribes to the SSE log stream for a job.
 *
 * Each connection replays the entire log from the start, then tails live —
 * there is no resume cursor. The browser auto-reconnects on transient drops
 * and re-replays, so we RESET every per-source buffer on the `open` event to
 * keep reconnects idempotent (no duplicated content). On `complete` we close
 * the EventSource for good and never reconnect.
 */
function useJobLogStream(jobId: string, owner: "job" | "sync") {
  const [buffers, setBuffers] = useState<Record<string, string>>({});
  const [sources, setSources] = useState<string[]>([]);
  const [status, setStatus] = useState<StreamStatus>("connecting");
  // Bumps whenever buffers are reset on (re)open so the viewer can decide to
  // snap back to the bottom for a fresh replay.
  const [resetToken, setResetToken] = useState(0);

  useEffect(() => {
    setBuffers({});
    setSources([]);
    setStatus("connecting");

    const resource =
      owner === "sync"
        ? `/api/v1/sync/operations/${encodeURIComponent(jobId)}`
        : `/api/v1/jobs/${encodeURIComponent(jobId)}`;
    const url = `${API_BASE}${resource}/logs/stream`;
    const es = new EventSource(url);
    let completed = false;

    const ensureSource = (prev: string[], source: string) =>
      prev.includes(source) ? prev : [...prev, source];

    es.addEventListener("open", () => {
      // Full-replay-on-open: drop everything and rebuild from this connection.
      setBuffers({});
      setResetToken((token) => token + 1);
      setStatus((current) => (current === "complete" ? current : "live"));
    });

    es.addEventListener("manifest", (event) => {
      try {
        const data = JSON.parse((event as MessageEvent).data) as {
          sources: string[];
        };
        setSources((prev) => {
          const next = [...prev];
          for (const source of data.sources ?? []) {
            if (!next.includes(source)) next.push(source);
          }
          return next;
        });
      } catch {
        // Ignore malformed manifest frames.
      }
    });

    es.addEventListener("append", (event) => {
      try {
        const data = JSON.parse((event as MessageEvent).data) as {
          source: string;
          text: string;
        };
        if (!data.source) return;
        setSources((prev) => ensureSource(prev, data.source));
        setBuffers((prev) => ({
          ...prev,
          [data.source]: (prev[data.source] ?? "") + (data.text ?? ""),
        }));
      } catch {
        // Ignore malformed append frames.
      }
    });

    es.addEventListener("complete", () => {
      completed = true;
      setStatus("complete");
      es.close();
    });

    es.addEventListener("error", () => {
      // EventSource auto-reconnects unless we've already completed. Surface a
      // subtle reconnecting state; only `complete` is terminal.
      if (!completed) {
        setStatus("reconnecting");
      }
    });

    return () => {
      es.close();
    };
  }, [jobId, owner]);

  return { buffers, sources, status, resetToken };
}

export default function TabbedLogViewer({ jobId, owner = "job" }: Props) {
  const { buffers, sources, status, resetToken } = useJobLogStream(jobId, owner);
  const [activeSource, setActiveSource] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const debouncedSearch = useDebounce(searchQuery, 150);
  const searchActive = debouncedSearch.trim().length > 0;
  const [logViewportHeight, setLogViewportHeight] = useState(() => {
    if (typeof window === "undefined") {
      return DESKTOP_LOG_VIEWPORT_HEIGHT;
    }
    return window.innerWidth < 768
      ? MOBILE_LOG_VIEWPORT_HEIGHT
      : DESKTOP_LOG_VIEWPORT_HEIGHT;
  });

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }
    const updateViewportHeight = () => {
      setLogViewportHeight(
        window.innerWidth < 768
          ? MOBILE_LOG_VIEWPORT_HEIGHT
          : DESKTOP_LOG_VIEWPORT_HEIGHT,
      );
    };
    updateViewportHeight();
    window.addEventListener("resize", updateViewportHeight);
    return () => window.removeEventListener("resize", updateViewportHeight);
  }, []);

  // Default the active tab to worker.log (the primary source) once tabs exist.
  useEffect(() => {
    setActiveSource((current) => {
      if (current && sources.includes(current)) {
        return current;
      }
      if (sources.includes(PRIMARY_SOURCE)) {
        return PRIMARY_SOURCE;
      }
      return sources[0] ?? null;
    });
  }, [sources]);

  const currentText = activeSource ? (buffers[activeSource] ?? "") : "";
  const isPrimary = activeSource === PRIMARY_SOURCE;

  const logLines = useMemo(() => splitLines(currentText), [currentText]);

  const displayLines = useMemo(() => {
    if (!searchActive) return logLines;
    const needle = debouncedSearch.trim().toLowerCase();
    return logLines.filter((line) => line.toLowerCase().includes(needle));
  }, [logLines, debouncedSearch, searchActive]);

  function handleDownload() {
    if (!activeSource) return;
    downloadLogText(buffers[activeSource] ?? "", activeSource);
  }

  if (status === "connecting" && sources.length === 0) {
    return (
      <div className="border border-edge bg-black px-5 py-8">
        <EmptyState>Connecting to log stream…</EmptyState>
      </div>
    );
  }

  return (
    <div className="space-y-0">
      <div className="border border-edge border-b-0 bg-surface-alt">
        <div className="flex flex-col md:flex-row md:items-center md:justify-between">
          <div className="min-w-0 overflow-x-auto border-b border-edge md:border-b-0">
            <div className="flex min-w-max">
              {sources.map((source) => {
                const shortLabel = source.split("/").pop() || source;
                return (
                  <button
                    key={source}
                    onClick={() => setActiveSource(source)}
                    title={source}
                    className={`shrink-0 whitespace-nowrap px-3 py-3 font-mono text-xs font-bold uppercase tracking-[0.15em] transition sm:px-5 ${
                      activeSource === source
                        ? "border-b border-success bg-black text-success"
                        : "text-soft hover:bg-black/50 hover:text-muted"
                    }`}
                  >
                    {shortLabel}
                  </button>
                );
              })}
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-3 px-4 py-3 sm:px-5">
            <StreamStatusBadge status={status} />
            <div className="relative flex w-full items-center sm:w-56">
              <FaIcon
                icon={faMagnifyingGlass}
                className="pointer-events-none absolute left-2 text-soft"
              />
              <input
                type="text"
                value={searchQuery}
                onChange={(event) => setSearchQuery(event.target.value)}
                placeholder="Filter lines…"
                aria-label="Filter log lines"
                className="w-full border border-edge bg-black py-1.5 pl-8 pr-8 font-mono text-xs text-white placeholder:text-soft outline-none focus:border-accent-lime"
              />
              {searchQuery ? (
                <button
                  type="button"
                  onClick={() => setSearchQuery("")}
                  aria-label="Clear filter"
                  className="absolute right-2 text-soft transition hover:text-white"
                >
                  <FaIcon icon={faXmark} />
                </button>
              ) : null}
            </div>
            {searchActive ? (
              <span className="font-mono text-[11px] uppercase tracking-[0.15em] text-soft">
                {displayLines.length} / {logLines.length} match
                {displayLines.length === 1 ? "" : "es"}
              </span>
            ) : null}
            <button
              onClick={handleDownload}
              disabled={!activeSource || currentText.length === 0}
              className="inline-flex w-full items-center justify-center font-mono text-xs font-bold uppercase tracking-[0.1em] text-soft transition hover:text-success disabled:cursor-not-allowed disabled:opacity-50 sm:w-auto"
            >
              <FaIcon icon={faDownload} className="mr-2" />
              Download
            </button>
          </div>
        </div>
      </div>

      <div className="border border-edge bg-black">
        {logLines.length === 0 ? (
          <div className="px-5 py-8">
            <EmptyState>No log content available yet.</EmptyState>
          </div>
        ) : searchActive && displayLines.length === 0 ? (
          <div className="px-5 py-8">
            <EmptyState>
              No log lines match{" "}
              <span className="font-mono text-white">
                "{debouncedSearch.trim()}"
              </span>
              .
            </EmptyState>
          </div>
        ) : isPrimary && !searchActive ? (
          <GroupedLogView
            sourcePath={activeSource ?? PRIMARY_SOURCE}
            lines={logLines}
            viewportHeight={logViewportHeight}
            resetToken={resetToken}
          />
        ) : (
          <VirtualizedAnsiLines
            sourcePath={`${activeSource ?? "unknown"}${searchActive ? ":filtered" : ""}`}
            lines={displayLines}
            viewportHeight={logViewportHeight}
            resetToken={resetToken}
          />
        )}
      </div>
    </div>
  );
}

function StreamStatusBadge({ status }: { status: StreamStatus }) {
  if (status === "complete") {
    return (
      <span className="inline-flex items-center gap-2 font-mono text-[11px] font-bold uppercase tracking-[0.15em] text-success">
        <span className="h-2 w-2 bg-success" />
        Complete
      </span>
    );
  }
  if (status === "reconnecting") {
    return (
      <span className="inline-flex items-center gap-2 font-mono text-[11px] font-bold uppercase tracking-[0.15em] text-accent-amber">
        <span className="h-2 w-2 animate-pulse bg-accent-amber" />
        Reconnecting…
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-2 font-mono text-[11px] font-bold uppercase tracking-[0.15em] text-accent-lime">
      <span className="h-2 w-2 animate-pulse bg-accent-lime" />
      Live
    </span>
  );
}

function splitLines(text: string): string[] {
  if (text.length === 0) return [];
  const lines = text.split("\n");
  if (lines.at(-1) === "") {
    lines.pop();
  }
  return lines;
}
