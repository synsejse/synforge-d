import { Link } from "@tanstack/react-router";
import { faBoxesStacked, faDownload } from "@fortawesome/free-solid-svg-icons";
import type { PublishedRepoFile } from "../../../lib/types";
import { formatBytes } from "../../../lib/bytes";
import { formatDateTime } from "../../../lib/datetime";
import Badge from "../../../components/ui/badge";
import FaIcon from "../../../components/ui/fa-icon";
import MetaPair from "../../../components/ui/meta-pair";
import Tooltip from "../../../components/ui/tooltip";

interface RepoFileCardProps {
  file: PublishedRepoFile;
  showPackageContext?: boolean;
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

export default function RepoFileCard({
  file,
  showPackageContext = false,
}: RepoFileCardProps) {
  const fileName = file.path.split("/").pop() || file.path;
  const signing = getSigningState(file);
  // SIGN FAILED overrides kind tint — destructive state should jump out.
  const accent =
    file.signing_status === "failed"
      ? "var(--theme-error-red)"
      : (KIND_RAIL[file.kind] ?? "var(--theme-text-soft)");
  const kindChip = KIND_CHIP[file.kind] ?? "border-edge-strong text-soft";

  return (
    <article className="relative border-2 border-edge-strong bg-black transition-colors">
      <span
        aria-hidden="true"
        className="absolute inset-y-0 left-0 w-1"
        style={{ background: accent }}
      />

      <div className="flex flex-col gap-2 pl-4 pr-4 py-2.5 sm:gap-3 sm:pl-6 sm:pr-5 sm:py-4 lg:flex-row lg:items-start lg:gap-4">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-x-2 gap-y-1.5 sm:gap-x-3">
            <a
              href={`/repo/${file.path}`}
              className="break-all font-mono text-sm font-bold text-white transition-colors hover:text-accent-lime sm:text-base"
            >
              {fileName}
            </a>
            <span
              className={`border-2 bg-black px-2 py-0.5 font-mono text-[10px] font-bold uppercase tracking-[0.18em] ${kindChip}`}
            >
              {file.kind}
            </span>
            <Badge variant={signing.variant} title={signing.title}>
              {signing.label}
            </Badge>
            {showPackageContext ? (
              <span className="border-2 border-edge-strong bg-black px-2 py-0.5 font-mono text-[10px] font-bold uppercase tracking-[0.14em] text-soft">
                {file.package_name}
              </span>
            ) : null}
            {showPackageContext && file.mock_chroot ? (
              <span className="border-2 border-edge bg-black px-2 py-0.5 font-mono text-[10px] font-bold uppercase tracking-[0.14em] text-soft">
                {file.mock_chroot}
              </span>
            ) : null}
          </div>

          <div className="mt-1 hidden break-all font-mono text-[11px] text-soft md:block">
            {file.path}
          </div>

          <div className="mt-1.5 flex flex-wrap items-start gap-x-6 gap-y-1.5 font-mono text-xs sm:mt-2 sm:gap-y-2">
            <MetaPair label="Size">
              <span className="text-strong">{formatBytes(file.size_bytes)}</span>
            </MetaPair>
            <MetaPair label="Published">
              <span className="text-strong">
                {formatDateTime(file.published_at)}
              </span>
            </MetaPair>
            <MetaPair label="Build">
              <Link
                to="/jobs/view"
                search={{ id: file.job_id }}
                className="break-all text-strong transition-colors hover:text-accent-lime"
              >
                <FaIcon icon={faBoxesStacked} className="mr-1.5 text-soft" />
                <span className="md:hidden">{file.job_id.slice(0, 8)}</span>
                <span className="hidden md:inline">{file.job_id}</span>
              </Link>
            </MetaPair>
          </div>
        </div>

        <div className="flex shrink-0 gap-1">
          <Tooltip content="Download" side="top">
            <a
              href={`/repo/${file.path}`}
              download
              aria-label={`Download ${fileName}`}
              className="inline-flex h-8 w-8 items-center justify-center border-2 border-edge-strong bg-transparent text-strong transition-colors hover:border-accent-lime hover:bg-surface-hover hover:text-accent-lime focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-lime"
            >
              <FaIcon icon={faDownload} />
            </a>
          </Tooltip>
        </div>
      </div>
    </article>
  );
}

function getSigningState(file: PublishedRepoFile) {
  if (file.signing_status === "signed") {
    return { label: "SIGNED", variant: "success" as const, title: undefined };
  }
  if (file.signing_status === "failed") {
    return {
      label: "SIGN FAILED",
      variant: "error" as const,
      title: file.signing_error_message || "Artifact signing failed",
    };
  }
  return { label: "NOT SIGNED", variant: "warning" as const, title: undefined };
}
