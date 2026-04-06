import Ansi from "ansi-to-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { faDownload } from "@fortawesome/free-solid-svg-icons";
import type { CSSProperties, UIEvent } from "react";

import api from "../../lib/api";
import { formatBytes } from "../../lib/bytes";
import type { LogManifestResponse } from "../../lib/types";
import EmptyState from "../ui/EmptyState";
import FaIcon from "../ui/FaIcon";
import LoadingBlock from "../ui/LoadingBlock";

interface Props {
  jobId: string;
  isLive: boolean;
}

interface LogState {
  text: string;
  startLine: number;
  cursor: number;
  loading: boolean;
  complete: boolean;
}

const POLL_INTERVAL_MS = 2000;
const CHUNK_SIZE = 65536;
const INITIAL_WINDOW_BYTES = 256 * 1024;
const ROW_HEIGHT = 24;
const LOG_VIEWPORT_HEIGHT = 600;

export default function TabbedLogViewer({ jobId, isLive }: Props) {
  const [manifest, setManifest] = useState<LogManifestResponse | null>(null);
  const [manifestLoading, setManifestLoading] = useState(true);
  const [activeSourcePath, setActiveSourcePath] = useState<string | null>(null);
  const [logStates, setLogStates] = useState<Record<string, LogState>>({});
  const [followLogs, setFollowLogs] = useState(true);
  const [downloading, setDownloading] = useState(false);
  const [pageVisible, setPageVisible] = useState(() => {
    if (typeof document === "undefined") {
      return true;
    }
    return document.visibilityState === "visible";
  });
  const pollingRef = useRef(false);
  const rawViewportRef = useRef<HTMLDivElement | null>(null);
  const logStatesRef = useRef<Record<string, LogState>>({});
  const scrollOffsetsRef = useRef<Record<string, number>>({});

  useEffect(() => {
    logStatesRef.current = logStates;
  }, [logStates]);

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

  const currentLog = activeSourcePath ? logStates[activeSourcePath] : null;
  const logLines = useMemo(() => {
    const text = currentLog?.text ?? "";
    const lines = text.split("\n");
    if (lines.at(-1) === "") {
      lines.pop();
    }
    return lines;
  }, [currentLog?.text]);

  async function loadManifest() {
    const response = await api.getJobLogManifest(jobId);
    setManifest(response);
    setActiveSourcePath((current) => {
      if (current) {
        return current;
      }
      return response.sources[0]?.file ?? null;
    });
    setManifestLoading(false);
  }

  async function loadLogChunk(sourcePath: string, reset = false) {
    const existing = logStatesRef.current[sourcePath] ?? {
      text: "",
      startLine: 1,
      cursor: 0,
      loading: false,
      complete: false,
    };
    if (existing.loading || (existing.complete && !reset && !isLive)) {
      return;
    }

    const cursor = reset ? undefined : existing.cursor;
    setLogStates((current) => ({
      ...current,
      [sourcePath]: { ...existing, loading: true },
    }));

    try {
      const response = reset
        ? await api.getJobLogChunk(jobId, {
            source: sourcePath,
            cursor: (await api.getJobLogMeta(jobId, sourcePath)).max_cursor,
            offset: -INITIAL_WINDOW_BYTES,
            limit: INITIAL_WINDOW_BYTES,
          })
        : await api.getJobLogChunk(jobId, {
            source: sourcePath,
            cursor,
            limit: CHUNK_SIZE,
          });
      setLogStates((current) => ({
        ...current,
        [sourcePath]: {
          text: reset
            ? response.contents
            : (current[sourcePath]?.text ?? "") + response.contents,
          startLine: reset
            ? response.start_line
            : (current[sourcePath]?.startLine ?? response.start_line),
          cursor: response.cursor,
          loading: false,
          complete: response.complete,
        },
      }));
    } catch {
      setLogStates((current) => ({
        ...current,
        [sourcePath]: { ...existing, loading: false },
      }));
    }
  }

  async function handleDownloadLog() {
    if (!activeSourcePath) {
      return;
    }
    setDownloading(true);
    try {
      await api.downloadJobLog(jobId, activeSourcePath);
    } finally {
      setDownloading(false);
    }
  }

  useEffect(() => {
    loadManifest().catch(() => {
      setManifestLoading(false);
    });
  }, [jobId]);

  useEffect(() => {
    if (!activeSourcePath) {
      return;
    }
    if (!currentLog || currentLog.text.length === 0) {
      void loadLogChunk(activeSourcePath, true);
    }
  }, [activeSourcePath, jobId, currentLog?.text]);

  useEffect(() => {
    if (!isLive || !followLogs || !activeSourcePath || !pageVisible) {
      return;
    }

    void (async () => {
      if (pollingRef.current) {
        return;
      }
      pollingRef.current = true;
      try {
        await loadManifest();
        await loadLogChunk(activeSourcePath);
      } finally {
        pollingRef.current = false;
      }
    })();

    const timer = window.setInterval(async () => {
      if (pollingRef.current) {
        return;
      }
      pollingRef.current = true;
      try {
        await loadManifest();
        await loadLogChunk(activeSourcePath);
      } finally {
        pollingRef.current = false;
      }
    }, POLL_INTERVAL_MS);

    return () => window.clearInterval(timer);
  }, [activeSourcePath, followLogs, isLive, pageVisible]);

  useEffect(() => {
    if (!followLogs) {
      return;
    }
    const viewport = rawViewportRef.current;
    if (!viewport) {
      return;
    }
    viewport.scrollTop = viewport.scrollHeight;
  }, [currentLog?.text, followLogs]);

  if (manifestLoading) {
    return <LoadingBlock label="Loading log sources…" lines={3} />;
  }

  return (
    <div className="space-y-0">
      {/* Controls Bar */}
      <div className="border-2 border-zinc-700 border-b-0 bg-black px-5 py-3 flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <label className="inline-flex items-center gap-2 font-mono text-xs uppercase tracking-[0.15em] text-zinc-400">
            <input
              type="checkbox"
              checked={followLogs}
              onChange={(event) => setFollowLogs(event.target.checked)}
            />
            Follow
          </label>
          <button
            onClick={handleDownloadLog}
            disabled={!activeSourcePath || downloading}
            className="font-mono text-xs font-bold uppercase tracking-[0.1em] text-zinc-500 transition hover:text-[var(--theme-terminal-green)] disabled:cursor-not-allowed disabled:opacity-50"
          >
            <FaIcon icon={faDownload} className="mr-2" />
            {downloading ? "Downloading…" : "Download"}
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-2 border-zinc-700 border-b-0 bg-zinc-950 flex flex-wrap">
        {(manifest?.sources ?? []).map((source) => (
          <button
            key={source.file}
            onClick={() => setActiveSourcePath(source.file)}
            className={`px-5 py-3 font-mono text-xs font-bold uppercase tracking-[0.15em] transition ${
              activeSourcePath === source.file
                ? "bg-black text-[var(--theme-terminal-green)] border-b-4 border-[var(--theme-terminal-green)]"
                : "text-zinc-500 hover:text-zinc-300 hover:bg-black/50"
            }`}
          >
            {source.file}
            {source.size > 0 && (
              <span className="ml-2 text-[10px] text-zinc-600">
                {formatBytes(source.size, "metric")}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Log Content */}
      <div className="border-2 border-zinc-700 bg-black">
        {logLines.length === 0 && !currentLog?.loading ? (
          <div className="px-5 py-8">
            <EmptyState>No log content available yet.</EmptyState>
          </div>
        ) : (
          <VirtualizedAnsiLines
            sourcePath={activeSourcePath ?? "unknown"}
            lines={logLines.length > 0 ? logLines : ["Waiting for output…"]}
            viewportRef={rawViewportRef}
            initialScrollTop={
              activeSourcePath
                ? (scrollOffsetsRef.current[activeSourcePath] ?? 0)
                : 0
            }
            onScrollTopChange={(nextScrollTop) => {
              if (activeSourcePath) {
                scrollOffsetsRef.current[activeSourcePath] = nextScrollTop;
              }
            }}
          />
        )}
      </div>
    </div>
  );
}

function VirtualizedAnsiLines({
  sourcePath,
  lines,
  viewportRef,
  initialScrollTop,
  onScrollTopChange,
}: {
  sourcePath: string;
  lines: string[];
  viewportRef: React.RefObject<HTMLDivElement | null>;
  initialScrollTop: number;
  onScrollTopChange: (scrollTop: number) => void;
}) {
  const [scrollTop, setScrollTop] = useState(0);

  useEffect(() => {
    setScrollTop(initialScrollTop);
    if (viewportRef.current) {
      viewportRef.current.scrollTop = initialScrollTop;
    }
  }, [initialScrollTop, sourcePath, viewportRef]);

  const totalHeight = lines.length * ROW_HEIGHT;
  const overscan = 12;
  const visibleStart = Math.max(
    0,
    Math.floor(scrollTop / ROW_HEIGHT) - overscan,
  );
  const visibleCount =
    Math.ceil(LOG_VIEWPORT_HEIGHT / ROW_HEIGHT) + overscan * 2;
  const visibleEnd = Math.min(lines.length, visibleStart + visibleCount);

  const visibleRows = useMemo(
    () =>
      lines.slice(visibleStart, visibleEnd).map((line, offset) => ({
        index: visibleStart + offset,
        line,
      })),
    [lines, visibleEnd, visibleStart],
  );

  function handleScroll(event: UIEvent<HTMLDivElement>) {
    const nextScrollTop = event.currentTarget.scrollTop;
    setScrollTop(nextScrollTop);
    onScrollTopChange(nextScrollTop);
  }

  return (
    <div
      ref={viewportRef}
      onScroll={handleScroll}
      className="max-h-[78vh] overflow-auto font-mono text-[13px] leading-6 bg-black"
      style={{ height: LOG_VIEWPORT_HEIGHT }}
    >
      <div
        style={{
          height: totalHeight || ROW_HEIGHT,
          position: "relative",
          minWidth: "max-content",
        }}
      >
        {visibleRows.map(({ index, line }) => {
          const style: CSSProperties = {
            position: "absolute",
            top: index * ROW_HEIGHT,
            left: 0,
            right: 0,
            height: ROW_HEIGHT,
          };

          return (
            <div
              key={`${sourcePath}:${index}`}
              style={style}
              className="px-5 text-[var(--theme-terminal-green)] whitespace-pre hover:bg-zinc-950/30"
            >
              <Ansi>{line || " "}</Ansi>
            </div>
          );
        })}
      </div>
    </div>
  );
}
