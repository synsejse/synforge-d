import { Link } from "@tanstack/react-router";
import { faDownload } from "@fortawesome/free-solid-svg-icons";
import type { PublishedRepoFile } from "../../../lib/types";
import { formatBytes } from "../../../lib/bytes";
import { formatDateTime } from "../../../lib/datetime";
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

interface RepoFileCardProps {
  file: PublishedRepoFile;
  /** When true, surface package name + target chroot in the header
      (used by the cross-package repository browser). */
  showPackageContext?: boolean;
}

export default function RepoFileCard({
  file,
  showPackageContext = false,
}: RepoFileCardProps) {
  const fileName = file.path.split("/").pop() || file.path;
  const rail = file.signing_status === "failed" ? ERROR_RAIL : ACCENT_RAIL;

  return (
    <RecordCard
      rail={rail}
      title={
        <a
          href={`/repo/${file.path}`}
          className="min-w-0 flex-1 break-all font-mono text-[13px] font-bold text-white transition-colors hover:text-accent-lime"
        >
          {fileName}
        </a>
      }
      badges={
        <>
          <KindBadge kind={file.kind} />
          <SigningBadge
            status={file.signing_status}
            errorMessage={file.signing_error_message}
          />
          {showPackageContext ? <RecordChip>{file.package_name}</RecordChip> : null}
          {showPackageContext && file.mock_chroot ? (
            <RecordChip>{file.mock_chroot}</RecordChip>
          ) : null}
        </>
      }
      actions={
        <Tooltip content="Download" side="top">
          <a
            href={`/repo/${file.path}`}
            download
            aria-label={`Download ${fileName}`}
            className={rowActionClass}
          >
            <FaIcon icon={faDownload} className="text-[13px]" />
          </a>
        </Tooltip>
      }
    >
      <div className="mt-2.5 break-all font-mono text-xs leading-[1.3] text-[#52525b]">
        {file.path}
      </div>
      <RecordMeta
        items={[
          { label: "Size", value: formatBytes(file.size_bytes) },
          { label: "Published", value: formatDateTime(file.published_at) },
          {
            label: "Build",
            value: (
              <Link
                to="/jobs/view"
                search={{ id: file.job_id }}
                className="text-[#52525b] transition-colors hover:text-accent-lime"
              >
                {file.job_id}
              </Link>
            ),
          },
        ]}
      />
    </RecordCard>
  );
}
