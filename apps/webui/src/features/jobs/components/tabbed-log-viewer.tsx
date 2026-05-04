import Ansi from "ansi-to-react";
import { useEffect, useMemo, useRef, useState } from "react";
import {
  faDownload,
  faMagnifyingGlass,
  faXmark,
} from "@fortawesome/free-solid-svg-icons";
import type { CSSProperties, UIEvent } from "react";

import api from "../../../lib/api";
import { formatBytes } from "../../../lib/bytes";
import type { LogManifestResponse } from "../../../lib/types";
import EmptyState from "../../../components/ui/empty-state";
import FaIcon from "../../../components/ui/fa-icon";
import { SkeletonTable } from "../../../components/ui/skeleton";
import { usePageVisible } from "../../../components/common/page-visibility-provider";
import { useDebounce } from "../../../lib/hooks/use-debounce";

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
const MOBILE_LOG_VIEWPORT_HEIGHT = 420;
const DESKTOP_LOG_VIEWPORT_HEIGHT = 600;

export default function TabbedLogViewer({ jobId, isLive }: Props) {
  const pageVisible = usePageVisible();
  const [manifest, setManifest] = useState<LogManifestResponse | null>(null);
  const [manifestLoading, setManifestLoading] = useState(true);
  const [activeSourcePath, setActiveSourcePath] = useState<string | null>(null);
  const [logStates, setLogStates] = useState<Record<string, LogState>>({});
  const [followLogs, setFollowLogs] = useState(true);
  const [downloading, setDownloading] = useState(false);
  const [logViewportHeight, setLogViewportHeight] = useState(() => {
    if (typeof window === "undefined") {
      return DESKTOP_LOG_VIEWPORT_HEIGHT;
    }
    return window.innerWidth < 768
      ? MOBILE_LOG_VIEWPORT_HEIGHT
      : DESKTOP_LOG_VIEWPORT_HEIGHT;
  });
  const [searchQuery, setSearchQuery] = useState("");
  const debouncedSearch = useDebounce(searchQuery, 150);
  const searchActive = debouncedSearch.trim().length > 0;
  const pollingRef = useRef(false);
  const rawViewportRef = useRef<HTMLDivElement | null>(null);
  const logStatesRef = useRef<Record<string, LogState>>({});
  const scrollOffsetsRef = useRef<Record<string, number>>({});

  useEffect(() => {
    logStatesRef.current = logStates;
  }, [logStates]);

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

  const currentLog = activeSourcePath ? logStates[activeSourcePath] : null;
  const logLines = useMemo(() => {
    const text = currentLog?.text ?? "";
    const lines = text.split("\n");
    if (lines.at(-1) === "") {
      lines.pop();
    }
    return lines;
  }, [currentLog?.text]);

  const displayLines = useMemo(() => {
    if (!searchActive) return logLines;
    const needle = debouncedSearch.trim().toLowerCase();
    return logLines.filter((line) => line.toLowerCase().includes(needle));
  }, [logLines, debouncedSearch, searchActive]);

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
    return <SkeletonTable columns={1} rows={6} />;
  }

  return (
    <div className="space-y-0">
      <div className="border-2 border-edge-strong border-b-0 bg-surface-alt">
        <div className="flex flex-col md:flex-row md:items-center md:justify-between">
          <div className="min-w-0 overflow-x-auto border-b-2 border-edge-strong md:border-b-0">
            <div className="flex min-w-max">
              {(manifest?.sources ?? []).map((source) => {
                const shortLabel = source.file.split("/").pop() || source.file;
                return (
                  <button
                    key={source.file}
                    onClick={() => setActiveSourcePath(source.file)}
                    title={source.file}
                    className={`shrink-0 whitespace-nowrap px-3 py-3 font-mono text-xs font-bold uppercase tracking-[0.15em] transition sm:px-5 ${
                      activeSourcePath === source.file
                        ? "border-b-4 border-success bg-black text-success"
                        : "text-soft hover:bg-black/50 hover:text-muted"
                    }`}
                  >
                    {shortLabel}
                    {source.size > 0 && (
                      <span className="ml-2 text-[10px] text-soft">
                        {formatBytes(source.size, "metric")}
                      </span>
                    )}
                  </button>
                );
              })}
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-3 px-4 py-3 sm:px-5">
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
                className="w-full border-2 border-edge-strong bg-black py-1.5 pl-8 pr-8 font-mono text-xs text-white placeholder:text-soft outline-none focus:border-accent-lime"
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
            <label className="inline-flex items-center gap-2 font-mono text-xs uppercase tracking-[0.15em] text-muted">
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
              className="inline-flex w-full items-center justify-center font-mono text-xs font-bold uppercase tracking-[0.1em] text-soft transition hover:text-success disabled:cursor-not-allowed disabled:opacity-50 sm:w-auto"
            >
              <FaIcon icon={faDownload} className="mr-2" />
              {downloading ? "Downloading…" : "Download"}
            </button>
          </div>
        </div>
      </div>

      <div className="border-2 border-edge-strong bg-black">
        {logLines.length === 0 && !currentLog?.loading ? (
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
        ) : (
          <VirtualizedAnsiLines
            sourcePath={`${activeSourcePath ?? "unknown"}${searchActive ? ":filtered" : ""}`}
            lines={displayLines.length > 0 ? displayLines : ["Waiting for output…"]}
            viewportRef={rawViewportRef}
            initialScrollTop={
              activeSourcePath && !searchActive
                ? (scrollOffsetsRef.current[activeSourcePath] ?? 0)
                : 0
            }
            onScrollTopChange={(nextScrollTop) => {
              if (activeSourcePath && !searchActive) {
                scrollOffsetsRef.current[activeSourcePath] = nextScrollTop;
              }
            }}
            viewportHeight={logViewportHeight}
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
  viewportHeight,
}: {
  sourcePath: string;
  lines: string[];
  viewportRef: React.RefObject<HTMLDivElement | null>;
  initialScrollTop: number;
  onScrollTopChange: (scrollTop: number) => void;
  viewportHeight: number;
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
    Math.ceil(viewportHeight / ROW_HEIGHT) + overscan * 2;
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
      className="max-h-[78vh] overflow-auto bg-black font-mono text-[13px] leading-6"
      style={{ height: viewportHeight }}
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
              className="whitespace-pre px-3 text-success hover:bg-surface-alt/30 sm:px-5"
            >
              <Ansi>{line || " "}</Ansi>
            </div>
          );
        })}
      </div>
    </div>
  );
}
