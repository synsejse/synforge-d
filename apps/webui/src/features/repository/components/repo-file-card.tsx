import { Link } from "@tanstack/react-router";
import { faDownload } from "@fortawesome/free-solid-svg-icons";
import type { PublishedRepoFile } from "../../../lib/types";
import { formatBytes } from "../../../lib/bytes";
import { formatDateTime } from "../../../lib/datetime";
import FaIcon from "../../../components/ui/fa-icon";
import Tooltip from "../../../components/ui/tooltip";

interface RepoFileCardProps {
  file: PublishedRepoFile;
  /** When true, surface package name + target chroot in the header
      (used by the cross-package repository browser). */
  showPackageContext?: boolean;
}

const KIND_CHIP: Record<string, string> = {
  rpm: "border-success text-success",
  srpm: "border-accent-orange text-accent-orange",
  debuginfo: "border-accent-cyan text-accent-cyan",
  debugsource: "border-accent-cyan text-accent-cyan",
  log: "border-edge-strong text-soft",
  other: "border-edge-strong text-muted",
};

export default function RepoFileCard({
  file,
  showPackageContext = false,
}: RepoFileCardProps) {
  const fileName = file.path.split("/").pop() || file.path;
  const signing = getSigningState(file);
  // SIGN FAILED overrides the amber rail — destructive state jumps out.
  const rail =
    file.signing_status === "failed"
      ? "var(--theme-error-red)"
      : "var(--theme-accent-lime)";
  const kindChip = KIND_CHIP[file.kind] ?? "border-edge-strong text-soft";

  return (
    <article className="sf-row relative border border-edge bg-black py-[15px] pl-[22px] pr-[18px] transition-colors hover:border-edge-strong hover:bg-[#0c0c0d]">
      <span
        aria-hidden="true"
        className="absolute inset-y-0 left-0 w-[2px]"
        style={{ background: rail }}
      />

      <div className="flex flex-wrap items-center gap-2.5">
        <a
          href={`/repo/${file.path}`}
          className="min-w-0 flex-1 break-all font-mono text-[13px] font-bold text-white transition-colors hover:text-accent-lime"
        >
          {fileName}
        </a>
        <span
          className={`shrink-0 border bg-black px-[7px] py-1 font-mono text-[9px] font-bold uppercase leading-none tracking-[0.08em] ${kindChip}`}
        >
          {file.kind}
        </span>
        <span
          className={`inline-flex shrink-0 items-center gap-1.5 border px-[7px] py-1 font-mono text-[9px] font-semibold uppercase leading-none tracking-[0.08em] ${signing.cls}`}
          title={signing.title}
        >
          <span aria-hidden="true" className={`h-[5px] w-[5px] ${signing.dot}`} />
          {signing.label}
        </span>
        {showPackageContext ? (
          <span className="shrink-0 border border-edge bg-black px-[7px] py-1 font-mono text-[9px] font-medium uppercase leading-none tracking-[0.04em] text-[#71717a]">
            {file.package_name}
          </span>
        ) : null}
        {showPackageContext && file.mock_chroot ? (
          <span className="shrink-0 border border-edge bg-black px-[7px] py-1 font-mono text-[9px] font-medium uppercase leading-none tracking-[0.04em] text-[#71717a]">
            {file.mock_chroot}
          </span>
        ) : null}
        <Tooltip content="Download" side="top">
          <a
            href={`/repo/${file.path}`}
            download
            aria-label={`Download ${fileName}`}
            className="sf-ic inline-flex h-[30px] w-[30px] shrink-0 items-center justify-center border border-edge bg-transparent text-soft transition-colors hover:border-accent-lime hover:text-accent-lime focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime"
          >
            <FaIcon icon={faDownload} className="text-[13px]" />
          </a>
        </Tooltip>
      </div>

      <div className="mt-2.5 break-all font-mono text-[11px] leading-[1.3] text-[#52525b]">
        {file.path}
      </div>

      <div className="mt-3 flex flex-wrap gap-x-8 gap-y-2">
        <FileMeta label="Size" value={formatBytes(file.size_bytes)} />
        <FileMeta label="Published" value={formatDateTime(file.published_at)} />
        <div>
          <span className="font-mono text-[9px] font-semibold uppercase tracking-[0.16em] text-[#6b6b73]">
            Build{" "}
          </span>
          <Link
            to="/jobs/view"
            search={{ id: file.job_id }}
            className="break-all font-mono text-[11px] text-[#52525b] transition-colors hover:text-accent-lime"
          >
            {file.job_id}
          </Link>
        </div>
      </div>
    </article>
  );
}

function FileMeta({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <span className="font-mono text-[9px] font-semibold uppercase tracking-[0.16em] text-[#6b6b73]">
        {label}{" "}
      </span>
      <span className="font-mono text-[11px] font-semibold text-muted">{value}</span>
    </div>
  );
}

function getSigningState(file: PublishedRepoFile) {
  if (file.signing_status === "signed") {
    return {
      label: "Signed",
      cls: "border-success text-success",
      dot: "bg-success",
      title: undefined as string | undefined,
    };
  }
  if (file.signing_status === "failed") {
    return {
      label: "Sign failed",
      cls: "border-error text-error",
      dot: "bg-error",
      title: file.signing_error_message || "Artifact signing failed",
    };
  }
  return {
    label: "Not signed",
    cls: "border-edge-strong text-soft",
    dot: "bg-soft",
    title: undefined as string | undefined,
  };
}
