import { faDownload } from "@fortawesome/free-solid-svg-icons";
import type { BuildArtifact } from "../../../lib/types";
import { API_BASE } from "../../../lib/api/client";
import { formatBytes } from "../../../lib/bytes";
import Badge from "../../../components/ui/badge";
import FaIcon from "../../../components/ui/fa-icon";
import MetaPair from "../../../components/ui/meta-pair";
import Tooltip from "../../../components/ui/tooltip";

interface ArtifactCardProps {
  jobId: string;
  artifact: BuildArtifact;
}

const KIND_RAIL: Record<string, string> = {
  rpm: "var(--theme-accent-lime)",
  srpm: "var(--theme-accent-orange)",
  debuginfo: "var(--theme-accent-cyan)",
  debugsource: "var(--theme-accent-cyan)",
  log: "var(--theme-text-soft)",
  other: "var(--theme-text-muted)",
};

const KIND_CHIP: Record<string, string> = {
  rpm: "border-accent-lime text-accent-lime",
  srpm: "border-accent-orange text-accent-orange",
  debuginfo: "border-accent-cyan text-accent-cyan",
  debugsource: "border-accent-cyan text-accent-cyan",
  log: "border-edge-strong text-soft",
  other: "border-edge-strong text-muted",
};

export default function ArtifactCard({ jobId, artifact }: ArtifactCardProps) {
  const fileName = artifact.file.split("/").pop() || artifact.file;
  const signing = getSigningState(artifact);
  // SIGN FAILED overrides kind tint — destructive state should jump out.
  const accent =
    artifact.signing_status === "failed"
      ? "var(--theme-error-red)"
      : (KIND_RAIL[artifact.kind] ?? "var(--theme-text-soft)");
  const kindChip = KIND_CHIP[artifact.kind] ?? "border-edge-strong text-soft";

  const downloadUrl = `${API_BASE}/api/v1/jobs/${encodeURIComponent(jobId)}/artifacts/${encodeArtifactPath(artifact.file)}/content`;

  return (
    <article className="relative border border-edge bg-black transition-colors">
      <span
        aria-hidden="true"
        className="absolute inset-y-0 left-0 w-1"
        style={{ background: accent }}
      />

      <div className="flex flex-col gap-3 pl-4 pr-4 py-3 sm:pl-6 sm:pr-5 sm:py-4 lg:flex-row lg:items-start lg:gap-4">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
            <a
              href={downloadUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="break-all font-mono text-sm font-bold text-white transition-colors hover:text-accent-lime sm:text-base"
            >
              {fileName}
            </a>
            <span
              className={`border-2 bg-black px-2 py-0.5 font-mono text-[10px] font-bold uppercase tracking-[0.18em] ${kindChip}`}
            >
              {artifact.kind}
            </span>
            <Badge variant={signing.variant} title={signing.title}>
              {signing.label}
            </Badge>
          </div>

          {artifact.file !== fileName ? (
            <div className="mt-1 break-all font-mono text-[11px] text-soft">
              {artifact.file}
            </div>
          ) : null}

          <div className="mt-2 flex flex-wrap items-start gap-x-6 gap-y-2 font-mono text-xs">
            <MetaPair label="Size">
              <span className="text-strong">
                {formatBytes(artifact.size_bytes)}
              </span>
            </MetaPair>
            <MetaPair label="Target">
              <span className="text-strong">{artifact.mock_chroot}</span>
            </MetaPair>
            <MetaPair label="SHA-256">
              <span className="break-all text-soft">
                {shortHash(artifact.sha256)}
              </span>
            </MetaPair>
          </div>
        </div>

        <div className="flex shrink-0 gap-1">
          <Tooltip content="Download" side="top">
            <a
              href={downloadUrl}
              target="_blank"
              rel="noopener noreferrer"
              aria-label={`Download ${fileName}`}
              className="inline-flex h-8 w-8 items-center justify-center border border-edge bg-transparent text-strong transition-colors hover:border-accent-lime hover:bg-surface-hover hover:text-accent-lime focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime"
            >
              <FaIcon icon={faDownload} />
            </a>
          </Tooltip>
        </div>
      </div>
    </article>
  );
}

function encodeArtifactPath(file: string): string {
  return file
    .split("/")
    .map((segment) => encodeURIComponent(segment))
    .join("/");
}

function shortHash(hash: string): string {
  if (!hash) return "—";
  return hash.length > 16 ? `${hash.slice(0, 8)}…${hash.slice(-8)}` : hash;
}

function getSigningState(artifact: BuildArtifact) {
  if (artifact.signing_status === "signed") {
    return { label: "SIGNED", variant: "success" as const, title: undefined };
  }
  if (artifact.signing_status === "failed") {
    return {
      label: "SIGN FAILED",
      variant: "error" as const,
      title: artifact.signing_error_message || "Artifact signing failed",
    };
  }
  return { label: "NOT SIGNED", variant: "warning" as const, title: undefined };
}
