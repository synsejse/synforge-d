import { faDownload, faFolderTree } from "@fortawesome/free-solid-svg-icons";
import { formatBytes } from "../../../lib/bytes";
import { formatDateTime } from "../../../lib/datetime";
import type { PublishedRepoFile } from "../../../lib/types";
import EmptyState from "../../../components/ui/empty-state";
import FaIcon from "../../../components/ui/fa-icon";
import Badge from "../../../components/ui/badge";

interface RepositoryRecentFilesSectionProps {
  recentFiles: PublishedRepoFile[];
}

export default function RepositoryRecentFilesSection({
  recentFiles,
}: RepositoryRecentFilesSectionProps) {
  const getSigningState = (file: PublishedRepoFile) => {
    if (file.signing_status === "signed") {
      return { label: "SIGNED", variant: "success" as const };
    }
    if (file.signing_status === "failed") {
      return {
        label: "SIGN FAILED",
        variant: "error" as const,
        title: file.signing_error_message || "Artifact signing failed",
      };
    }
    return { label: "NOT SIGNED", variant: "warning" as const };
  };

  return (
    <section className="border-2 border-zinc-700 bg-black p-6">
      <div className="flex items-end justify-between gap-4">
        <div>
          <div className="font-mono text-xs uppercase tracking-[0.22em] text-zinc-500">
            Recent Output
          </div>
          <h2 className="mt-2 font-mono text-xl font-bold uppercase text-white">
            Latest Published Files
          </h2>
        </div>
        <FaIcon icon={faFolderTree} className="text-zinc-500" />
      </div>

      {recentFiles.length === 0 ? (
        <div className="mt-5">
          <EmptyState>No published files have been recorded yet.</EmptyState>
        </div>
      ) : (
        <div className="mt-5 grid gap-3">
          {recentFiles.slice(0, 6).map((file) => {
            const signingState = getSigningState(file);
            return (
              <div
                key={`${file.job_id}:${file.path}`}
                className="grid gap-3 border-2 border-zinc-700 bg-zinc-950/40 p-4 md:grid-cols-[minmax(0,1fr)_auto]"
              >
                <div className="min-w-0">
                  <div className="truncate font-mono text-sm text-white">
                    {file.path}
                  </div>
                  <div className="mt-2 flex flex-wrap gap-2 font-mono text-xs uppercase tracking-[0.18em] text-zinc-500">
                    <span>{file.package_name}</span>
                    <span>{file.kind}</span>
                    <span>{formatBytes(file.size_bytes)}</span>
                  </div>
                  <div className="mt-2">
                    <Badge variant={signingState.variant} title={signingState.title}>
                      {signingState.label}
                    </Badge>
                  </div>
                  <div className="mt-2 font-mono text-xs text-zinc-500">
                    {formatDateTime(file.published_at)}
                  </div>
                </div>
                <div className="flex items-start md:justify-end">
                  <a
                    href={`/repo/${file.path}`}
                    className="inline-flex items-center border-2 border-zinc-700 bg-black px-3 py-2 font-mono text-xs font-bold uppercase tracking-[0.1em] text-zinc-200 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
                  >
                    <FaIcon icon={faDownload} className="mr-2" />
                    Download
                  </a>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </section>
  );
}
