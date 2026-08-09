import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type UIEvent,
} from "react";
import { faChevronDown, faChevronRight } from "@fortawesome/free-solid-svg-icons";
import AnsiText from "../../../components/ui/ansi-text";
import FaIcon from "../../../components/ui/fa-icon";

const GROUP_HEADER_RE = /^##\[group\]\s?(.*)$/;
const ROW_HEIGHT = 24;
const SCROLL_BOTTOM_THRESHOLD = 24;

interface LogViewProps {
  sourcePath: string;
  lines: string[];
  viewportHeight: number;
  resetToken: number;
}

function JumpToLive({ onClick }: { onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="absolute bottom-3 right-3 z-10 inline-flex items-center gap-2 border border-accent-lime bg-accent-lime px-3 py-1.5 font-mono text-xs font-bold uppercase tracking-[0.15em] text-black transition-[filter] hover:brightness-110"
    >
      <FaIcon icon={faChevronDown} />
      Jump to live
    </button>
  );
}

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

  useEffect(() => {
    setFollowing(true);
    const viewport = viewportRef.current;
    if (viewport) {
      viewport.scrollTop = viewport.scrollHeight;
    }
  }, [followKey, resetToken, viewportRef]);

  return { following, scrollToBottom, handleScroll };
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

function LogLine({ line }: { line: string }) {
  return (
    <div className="whitespace-pre px-3 text-success hover:bg-surface-alt/30 sm:px-5">
      <AnsiText>{line || " "}</AnsiText>
    </div>
  );
}

export function GroupedLogView({
  sourcePath,
  lines,
  viewportHeight,
  resetToken,
}: LogViewProps) {
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const [collapsed, setCollapsed] = useState<Record<number, boolean>>({});
  const { following, scrollToBottom, handleScroll } = useAutoFollow(
    viewportRef,
    sourcePath,
    resetToken,
  );
  const sections = useMemo(() => buildSections(lines), [lines]);

  useEffect(() => {
    if (!following) return;
    const viewport = viewportRef.current;
    if (viewport) viewport.scrollTop = viewport.scrollHeight;
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
        <div className="py-3" style={{ minWidth: "max-content" }}>
          {sections.map((section) => {
            if (section.title === null) {
              return section.lines.map((line, offset) => (
                <LogLine key={`${section.startLine}:${offset}`} line={line} />
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
                  <span className="ml-auto text-xs text-soft">
                    {section.lines.length}
                  </span>
                </button>
                {isCollapsed
                  ? null
                  : section.lines.map((line, offset) => (
                      <LogLine key={`${section.startLine}:${offset}`} line={line} />
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

export function VirtualizedAnsiLines({
  sourcePath,
  lines,
  viewportHeight,
  resetToken,
}: LogViewProps) {
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const { following, scrollToBottom, handleScroll } = useAutoFollow(
    viewportRef,
    sourcePath,
    resetToken,
  );

  const renderLines = useMemo(
    () => (lines.length > 0 ? lines : ["Waiting for output…"]),
    [lines],
  );
  const totalHeight = renderLines.length * ROW_HEIGHT;
  const overscan = 12;
  const visibleStart = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - overscan);
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

  useEffect(() => {
    if (!following) return;
    const viewport = viewportRef.current;
    if (viewport) viewport.scrollTop = viewport.scrollHeight;
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
        className="max-h-[78vh] overflow-auto bg-black pt-3 font-mono text-[13px] leading-6"
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
                <AnsiText>{line || " "}</AnsiText>
              </div>
            );
          })}
        </div>
      </div>
      {following ? null : <JumpToLive onClick={scrollToBottom} />}
    </div>
  );
}
