import { useEffect, useRef, useState } from "react";
import { Link } from "@tanstack/react-router";
import type { BuildJobResponse, JobResourceUsageSample } from "../../lib/types";
import { API_BASE } from "../../lib/api/client";
import { formatBytes } from "../../lib/bytes";

interface InFlightCardProps {
  entry: BuildJobResponse;
  usage: JobResourceUsageSample | null;
  now: number;
}

/**
 * Detailed live-build card for the dashboard "Builds in flight" panel —
 * package + phase header, CPU/MEM meters from the live resource sample, and
 * a tail of the streaming log. Mirrors the design comp.
 */
export default function InFlightCard({ entry, usage, now }: InFlightCardProps) {
  const job = entry.job;
  const logLines = useJobLogTail(job.id);

  const cpuPct = usage ? clamp((usage.cpu_percent / (Math.max(1, usage.online_cpus) * 100)) * 100) : 0;
  const memPct =
    usage && usage.memory_limit_bytes > 0
      ? clamp((usage.memory_usage_bytes / usage.memory_limit_bytes) * 100)
      : 0;
  const elapsed = formatElapsed(
    Math.max(0, now - Date.parse(job.started_at ?? job.created_at)),
  );

  return (
    <Link
      to="/jobs/view"
      search={{ id: job.id }}
      className="synforge-row-live block border border-edge bg-[#070708] p-[14px_16px] transition-colors hover:bg-[#0c0c0d]"
    >
      <div className="flex flex-wrap items-center gap-3">
        <span className="font-mono text-[14px] font-bold leading-none text-white">
          {job.package_name}
        </span>
        <span className="border border-edge px-[7px] py-1 font-mono text-[9px] font-medium uppercase leading-none tracking-[0.06em] text-muted">
          {job.mock_chroot}
        </span>
        <span className="font-mono text-[10px] font-semibold uppercase leading-none tracking-[0.08em] text-accent-lime">
          Running
        </span>
        <span className="ml-auto font-mono text-[11px] font-semibold leading-none tabular-nums text-soft">
          {elapsed}
        </span>
      </div>

      <div className="mt-3 grid grid-cols-2 gap-4">
        <Meter
          label="CPU"
          value={usage ? `${Math.round(usage.cpu_percent)}%` : "—"}
          valueClass="text-accent-lime"
          fillClass="bg-accent-lime"
          pct={cpuPct}
        />
        <Meter
          label="MEM"
          value={usage ? formatBytes(usage.memory_usage_bytes) : "—"}
          valueClass="text-accent-cyan"
          fillClass="bg-accent-cyan"
          pct={memPct}
        />
      </div>

      <div className="mt-3 max-h-[92px] overflow-hidden border border-[#161618] bg-black px-[11px] py-[9px] font-mono text-[11px] leading-[1.6] text-[#71717a]">
        {logLines.length > 0 ? (
          logLines.map((line, i) => (
            <div key={i} className="truncate">
              {line}
            </div>
          ))
        ) : (
          <div className="text-[#52525b]">Waiting for log output…</div>
        )}
      </div>
    </Link>
  );
}

function Meter({
  label,
  value,
  valueClass,
  fillClass,
  pct,
}: {
  label: string;
  value: string;
  valueClass: string;
  fillClass: string;
  pct: number;
}) {
  return (
    <div>
      <div className="flex items-center justify-between font-mono text-[9px] font-semibold uppercase leading-none tracking-[0.14em] text-soft">
        <span>{label}</span>
        <span className={`tabular-nums ${valueClass}`}>{value}</span>
      </div>
      <div className="mt-1.5 h-1.5 border border-edge bg-[#161618]">
        <div className={`h-full ${fillClass}`} style={{ width: `${pct}%` }} />
      </div>
    </div>
  );
}

function clamp(v: number): number {
  if (!Number.isFinite(v)) return 0;
  return Math.max(0, Math.min(100, v));
}

function formatElapsed(ms: number): string {
  const total = Math.floor(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h > 0) return `${h}h ${m}m ${s}s`;
  return `${m}m ${s}s`;
}

const MAX_TAIL_LINES = 6;
const MAX_BUFFER = 16_000;

/**
 * Lightweight tail of a job's SSE log stream — keeps only the most recent
 * lines for the dashboard preview. The stream full-replays on open; we trim
 * aggressively so memory stays bounded across active builds.
 */
function useJobLogTail(jobId: string): string[] {
  const [lines, setLines] = useState<string[]>([]);
  const bufferRef = useRef("");

  useEffect(() => {
    bufferRef.current = "";
    setLines([]);
    const url = `${API_BASE}/api/v1/jobs/${encodeURIComponent(jobId)}/logs/stream`;
    const es = new EventSource(url);

    const reset = () => {
      bufferRef.current = "";
      setLines([]);
    };
    es.addEventListener("open", reset);
    es.addEventListener("append", (event) => {
      try {
        const data = JSON.parse((event as MessageEvent).data) as { text?: string };
        if (!data.text) return;
        let buf = bufferRef.current + data.text;
        if (buf.length > MAX_BUFFER) buf = buf.slice(-MAX_BUFFER);
        bufferRef.current = buf;
        const tail = buf
          .split("\n")
          .filter((l) => l.trim().length > 0)
          .slice(-MAX_TAIL_LINES);
        setLines(tail);
      } catch {
        /* ignore malformed frames */
      }
    });
    es.addEventListener("complete", () => es.close());

    return () => es.close();
  }, [jobId]);

  return lines;
}
