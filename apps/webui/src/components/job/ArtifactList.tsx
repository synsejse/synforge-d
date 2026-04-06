import { faDownload } from "@fortawesome/free-solid-svg-icons";
import { formatBytes } from "../../lib/bytes";
import type { BuildArtifact } from "../../lib/types";
import EmptyState from "../ui/EmptyState";
import FaIcon from "../ui/FaIcon";
import Badge from "../ui/Badge";

interface ArtifactListProps {
  artifacts: BuildArtifact[];
  downloadingArtifactPath: string | null;
  onDownload: (artifact: BuildArtifact) => void;
}

export default function ArtifactList({
  artifacts,
  downloadingArtifactPath,
  onDownload,
}: ArtifactListProps) {
  const getSigningState = (artifact: BuildArtifact) => {
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

  return (
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
          {artifacts.map((artifact) => {
            const signingState = getSigningState(artifact);
            return (
              <div
                key={`${artifact.id}-${artifact.file}`}
                className="grid gap-3 border border-zinc-800 bg-black px-4 py-4 md:grid-cols-[minmax(0,1fr)_auto_auto_auto_auto]"
              >
                <div>
                  <div className="font-mono text-sm text-white">{artifact.file}</div>
                  <div className="mt-1 text-xs text-zinc-500">{artifact.sha256}</div>
                </div>
                <div className="text-sm text-zinc-300">
                  {formatBytes(artifact.size_bytes, "metric")}
                </div>
                <div className="text-sm uppercase tracking-[0.18em] text-zinc-500">
                  {artifact.kind}
                </div>
                <div>
                  <Badge variant={signingState.variant} title={signingState.title}>
                    {signingState.label}
                  </Badge>
                </div>
                <div className="flex md:justify-end">
                  <button
                    onClick={() => onDownload(artifact)}
                    disabled={downloadingArtifactPath === artifact.file}
                    className="inline-flex items-center border border-zinc-800 bg-black px-3 py-1.5 text-xs text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    <FaIcon icon={faDownload} className="mr-2 text-[0.95em]" />
                    {downloadingArtifactPath === artifact.file
                      ? "Downloading…"
                      : "Download"}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </section>
  );
}
