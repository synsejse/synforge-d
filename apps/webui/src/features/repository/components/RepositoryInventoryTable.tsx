import { faDownload } from "@fortawesome/free-solid-svg-icons";
import { formatBytes } from "../../../lib/bytes";
import { formatDateTime } from "../../../lib/datetime";
import type { PublishedRepoFile } from "../../../lib/types";
import EmptyState from "../../../components/ui/EmptyState";
import FaIcon from "../../../components/ui/FaIcon";
import Badge from "../../../components/ui/Badge";

interface RepositoryInventoryTableProps {
  files: PublishedRepoFile[];
}

export default function RepositoryInventoryTable({
  files,
}: RepositoryInventoryTableProps) {
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

  if (files.length === 0) {
    return <EmptyState>No managed repository files are published yet.</EmptyState>;
  }

  return (
    <div className="overflow-x-auto border-2 border-zinc-700 bg-black">
      <table className="w-full min-w-[640px] lg:min-w-[980px]">
        <caption className="sr-only">
          Published repository files with package, target, type, size, publication
          date, and actions.
        </caption>
        <thead className="border-b-2 border-zinc-700 bg-zinc-950 text-left font-mono text-xs uppercase tracking-[0.2em] text-zinc-500">
          <tr>
            <th scope="col" className="px-4 py-3">
              Package
            </th>
            <th scope="col" className="px-4 py-3">
              Repo Path
            </th>
            <th scope="col" className="px-4 py-3">
              Target
            </th>
            <th scope="col" className="px-4 py-3">
              Kind
            </th>
            <th scope="col" className="px-4 py-3">
              Size
            </th>
            <th scope="col" className="px-4 py-3">
              Published
            </th>
            <th scope="col" className="px-4 py-3">
              Signing
            </th>
            <th scope="col" className="px-4 py-3">
              Actions
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-zinc-800">
          {files.map((file) => {
            const signingState = getSigningState(file);
            return (
              <tr key={`${file.job_id}:${file.path}`} className="hover:bg-zinc-950">
                <td className="px-4 py-3">
                  <a
                    href={`/packages/view/?name=${encodeURIComponent(file.package_name)}`}
                    className="font-mono text-sm text-white transition hover:text-[var(--theme-accent-lime)]"
                  >
                    {file.package_name}
                  </a>
                </td>
                <td className="px-4 py-3 font-mono text-sm text-zinc-200">
                  {file.path}
                </td>
                <td className="px-4 py-3 font-mono text-sm text-zinc-400">
                  {file.mock_chroot || "unknown"}
                </td>
                <td className="px-4 py-3 font-mono text-xs uppercase tracking-[0.18em] text-zinc-500">
                  {file.kind}
                </td>
                <td className="px-4 py-3 font-mono text-sm text-zinc-400">
                  {formatBytes(file.size_bytes)}
                </td>
                <td className="px-4 py-3 font-mono text-sm text-zinc-400">
                  {formatDateTime(file.published_at)}
                </td>
                <td className="px-4 py-3">
                  <Badge variant={signingState.variant} title={signingState.title}>
                    {signingState.label}
                  </Badge>
                </td>
                <td className="px-4 py-3">
                  <div className="flex flex-wrap gap-2">
                    <a
                      href={`/repo/${file.path}`}
                      className="inline-flex items-center border-2 border-zinc-700 bg-black px-3 py-2 font-mono text-xs font-bold uppercase tracking-[0.1em] text-zinc-200 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
                    >
                      <FaIcon icon={faDownload} className="mr-2" />
                      Download
                    </a>
                    <a
                      href={`/jobs/view/?id=${encodeURIComponent(file.job_id)}`}
                      className="inline-flex items-center border-2 border-zinc-700 bg-black px-3 py-2 font-mono text-xs font-bold uppercase tracking-[0.1em] text-zinc-400 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950 hover:text-zinc-200"
                    >
                      Build
                    </a>
                  </div>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
