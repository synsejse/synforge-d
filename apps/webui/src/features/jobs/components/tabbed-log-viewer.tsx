import Ansi from "ansi-to-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  faChevronDown,
  faChevronRight,
  faDownload,
  faMagnifyingGlass,
  faXmark,
} from "@fortawesome/free-solid-svg-icons";
import type { CSSProperties, UIEvent } from "react";

import { API_BASE } from "../../../lib/api/client";
import { downloadLogText } from "../../../lib/api/jobs";
import EmptyState from "../../../components/ui/empty-state";
import FaIcon from "../../../components/ui/fa-icon";
import { useDebounce } from "../../../lib/hooks/use-debounce";

interface Props {
  jobId: string;
}

type StreamStatus = "connecting" | "live" | "reconnecting" | "complete";

const PRIMARY_SOURCE = "worker.log";
const GROUP_HEADER_RE = /^##\[group\]\s?(.*)$/;
const ROW_HEIGHT = 24;
const MOBILE_LOG_VIEWPORT_HEIGHT = 420;
const DESKTOP_LOG_VIEWPORT_HEIGHT = 600;
const SCROLL_BOTTOM_THRESHOLD = 24;

/**
 * Subscribes to the SSE log stream for a job.
 *
 * Each connection replays the entire log from the start, then tails live —
 * there is no resume cursor. The browser auto-reconnects on transient drops
 * and re-replays, so we RESET every per-source buffer on the `open` event to
 * keep reconnects idempotent (no duplicated content). On `complete` we close
 * the EventSource for good and never reconnect.
 */
function useJobLogStream(jobId: string) {
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

    const url = `${API_BASE}/api/v1/jobs/${encodeURIComponent(jobId)}/logs/stream`;
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
  }, [jobId]);

  return { buffers, sources, status, resetToken };
}

export default function TabbedLogViewer({ jobId }: Props) {
  const { buffers, sources, status, resetToken } = useJobLogStream(jobId);
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

function JumpToLive({ onClick }: { onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="absolute bottom-3 right-3 z-10 inline-flex items-center gap-2 border border-edge bg-accent-lime px-3 py-1.5 font-mono text-[11px] font-bold uppercase tracking-[0.15em] text-black shadow-brutal-sm transition hover:translate-y-px"
    >
      <FaIcon icon={faChevronDown} />
      Jump to live
    </button>
  );
}

/**
 * Tracks whether a scroll container is pinned to the bottom and exposes the
 * follow state plus a handler to (re)engage it. Auto-follow re-engages when the
 * user scrolls back to the bottom and disengages when they scroll up.
 */
function useAutoFollow(
  viewportRef: React.RefObject<HTMLDivElement | null>,
  followKey: string,
  resetToken: number,
) {
  const [following, setFollowing] = useState(true);

  const scrollToBottom = useCallback(() => {
    const viewport = viewportRef.current;
    if (!viewport) return;
    viewport.scrollTop = viewport.scrollHeight;
    setFollowing(true);
  }, [viewportRef]);

  const handleScroll = useCallback(() => {
    const viewport = viewportRef.current;
    if (!viewport) return;
    const distanceFromBottom =
      viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight;
    setFollowing(distanceFromBottom <= SCROLL_BOTTOM_THRESHOLD);
  }, [viewportRef]);

  // Re-engage follow when switching sources or on a fresh replay.
  useEffect(() => {
    setFollowing(true);
    const viewport = viewportRef.current;
    if (viewport) {
      viewport.scrollTop = viewport.scrollHeight;
    }
  }, [followKey, resetToken, viewportRef]);

  return { following, scrollToBottom, handleScroll, setFollowing };
}

interface Section {
  title: string | null;
  startLine: number;
  lines: string[];
}

function buildSections(lines: string[]): Section[] {
  const sections: Section[] = [];
  let current: Section | null = null;
  lines.forEach((line, index) => {
    const match = GROUP_HEADER_RE.exec(line);
    if (match) {
      current = { title: match[1] || "Section", startLine: index, lines: [] };
      sections.push(current);
      return;
    }
    if (!current) {
      current = { title: null, startLine: index, lines: [] };
      sections.push(current);
    }
    current.lines.push(line);
  });
  return sections;
}

/**
 * worker.log view with collapsible `##[group] {title}` sections. Sections
 * default to expanded so live tailing reads naturally. Lines before the first
 * group render ungrouped.
 */
function GroupedLogView({
  sourcePath,
  lines,
  viewportHeight,
  resetToken,
}: {
  sourcePath: string;
  lines: string[];
  viewportHeight: number;
  resetToken: number;
}) {
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const [collapsed, setCollapsed] = useState<Record<number, boolean>>({});
  const { following, scrollToBottom, handleScroll } = useAutoFollow(
    viewportRef,
    sourcePath,
    resetToken,
  );

  const sections = useMemo(() => buildSections(lines), [lines]);

  // Stick to the bottom as new content arrives while following.
  useEffect(() => {
    if (!following) return;
    const viewport = viewportRef.current;
    if (viewport) {
      viewport.scrollTop = viewport.scrollHeight;
    }
  }, [lines, collapsed, following]);

  const toggle = (key: number) =>
    setCollapsed((prev) => ({ ...prev, [key]: !prev[key] }));

  return (
    <div className="relative">
      <div
        ref={viewportRef}
        onScroll={handleScroll}
        className="max-h-[78vh] overflow-auto bg-black font-mono text-[13px] leading-6"
        style={{ height: viewportHeight }}
      >
        <div style={{ minWidth: "max-content" }}>
          {sections.map((section) => {
            if (section.title === null) {
              return section.lines.map((line, offset) => (
                <LogLine
                  key={`${section.startLine}:${offset}`}
                  line={line}
                />
              ));
            }
            const isCollapsed = collapsed[section.startLine] ?? false;
            return (
              <div key={section.startLine}>
                <button
                  type="button"
                  onClick={() => toggle(section.startLine)}
                  className="flex w-full items-center gap-2 border-y border-edge-strong bg-surface-alt/40 px-3 py-1 text-left font-mono text-xs font-bold uppercase tracking-[0.12em] text-muted transition hover:bg-surface-alt/70 hover:text-white sm:px-5"
                >
                  <FaIcon
                    icon={isCollapsed ? faChevronRight : faChevronDown}
                    className="text-soft"
                  />
                  <span className="truncate">{section.title}</span>
                  <span className="ml-auto text-[10px] text-soft">
                    {section.lines.length}
                  </span>
                </button>
                {isCollapsed
                  ? null
                  : section.lines.map((line, offset) => (
                      <LogLine
                        key={`${section.startLine}:${offset}`}
                        line={line}
                      />
                    ))}
              </div>
            );
          })}
        </div>
      </div>
      {following ? null : <JumpToLive onClick={scrollToBottom} />}
    </div>
  );
}

function LogLine({ line }: { line: string }) {
  return (
    <div className="whitespace-pre px-3 text-success hover:bg-surface-alt/30 sm:px-5">
      <Ansi>{line || " "}</Ansi>
    </div>
  );
}

function VirtualizedAnsiLines({
  sourcePath,
  lines,
  viewportHeight,
  resetToken,
}: {
  sourcePath: string;
  lines: string[];
  viewportHeight: number;
  resetToken: number;
}) {
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const { following, scrollToBottom, handleScroll } = useAutoFollow(
    viewportRef,
    sourcePath,
    resetToken,
  );

  const renderLines = lines.length > 0 ? lines : ["Waiting for output…"];
  const totalHeight = renderLines.length * ROW_HEIGHT;
  const overscan = 12;
  const visibleStart = Math.max(
    0,
    Math.floor(scrollTop / ROW_HEIGHT) - overscan,
  );
  const visibleCount = Math.ceil(viewportHeight / ROW_HEIGHT) + overscan * 2;
  const visibleEnd = Math.min(renderLines.length, visibleStart + visibleCount);

  const visibleRows = useMemo(
    () =>
      renderLines.slice(visibleStart, visibleEnd).map((line, offset) => ({
        index: visibleStart + offset,
        line,
      })),
    [renderLines, visibleEnd, visibleStart],
  );

  // Stick to the bottom as new content arrives while following.
  useEffect(() => {
    if (!following) return;
    const viewport = viewportRef.current;
    if (viewport) {
      viewport.scrollTop = viewport.scrollHeight;
    }
  }, [lines, following]);

  function onScroll(event: UIEvent<HTMLDivElement>) {
    setScrollTop(event.currentTarget.scrollTop);
    handleScroll();
  }

  return (
    <div className="relative">
      <div
        ref={viewportRef}
        onScroll={onScroll}
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
      {following ? null : <JumpToLive onClick={scrollToBottom} />}
    </div>
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
