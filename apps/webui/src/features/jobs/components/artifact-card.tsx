import { faDownload } from "@fortawesome/free-solid-svg-icons";
import type { BuildArtifact } from "../../../lib/types";
import { API_BASE } from "../../../lib/api/client";
import { formatBytes } from "../../../lib/bytes";
import FaIcon from "../../../components/ui/fa-icon";
import Tooltip from "../../../components/ui/tooltip";
import {
  ACCENT_RAIL,
  ERROR_RAIL,
  rowActionClass,
} from "../../../components/ui/record-card-styles";
import {
  KindBadge,
  RecordCard,
  RecordChip,
  RecordMeta,
  SigningBadge,
} from "../../../components/ui/record-card";

interface ArtifactCardProps {
  jobId: string;
  artifact: BuildArtifact;
}

export default function ArtifactCard({ jobId, artifact }: ArtifactCardProps) {
  const fileName = artifact.file.split("/").pop() || artifact.file;
  const rail = artifact.signing_status === "failed" ? ERROR_RAIL : ACCENT_RAIL;
  const downloadUrl = `${API_BASE}/api/v1/jobs/${encodeURIComponent(jobId)}/artifacts/${encodeArtifactPath(artifact.file)}/content`;

  return (
    <RecordCard
      rail={rail}
      title={
        <a
          href={downloadUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="min-w-0 flex-1 break-all font-mono text-[13px] font-bold text-white transition-colors hover:text-accent-lime"
        >
          {fileName}
        </a>
      }
      badges={
        <>
          <KindBadge kind={artifact.kind} />
          <SigningBadge
            status={artifact.signing_status}
            errorMessage={artifact.signing_error_message}
          />
          <RecordChip>{artifact.mock_chroot}</RecordChip>
        </>
      }
      actions={
        <Tooltip content="Download" side="top">
          <a
            href={downloadUrl}
            target="_blank"
            rel="noopener noreferrer"
            aria-label={`Download ${fileName}`}
            className={rowActionClass}
          >
            <FaIcon icon={faDownload} className="text-[13px]" />
          </a>
        </Tooltip>
      }
    >
      {artifact.file !== fileName ? (
        <div className="mt-2.5 break-all font-mono text-xs leading-[1.3] text-[#52525b]">
          {artifact.file}
        </div>
      ) : null}
      <RecordMeta
        items={[
          { label: "Size", value: formatBytes(artifact.size_bytes) },
          {
            label: "SHA-256",
            value: <span className="text-[#52525b]">{shortHash(artifact.sha256)}</span>,
          },
        ]}
      />
    </RecordCard>
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
