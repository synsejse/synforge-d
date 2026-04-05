import { faDownload } from "@fortawesome/free-solid-svg-icons";
import { formatBytes } from "../../lib/bytes";
import { formatDateTime } from "../../lib/datetime";
import type { PublishedRepoFile } from "../../lib/types";
import EmptyState from "../ui/EmptyState";
import FaIcon from "../ui/FaIcon";

interface RepositoryInventoryTableProps {
  files: PublishedRepoFile[];
}

export default function RepositoryInventoryTable({
  files,
}: RepositoryInventoryTableProps) {
  if (files.length === 0) {
    return <EmptyState>No managed repository files are published yet.</EmptyState>;
  }

  return (
    <div className="overflow-x-auto border border-zinc-800 bg-black">
      <table className="min-w-[980px] w-full">
        <caption className="sr-only">
          Published repository files with package, target, type, size, publication
          date, and actions.
        </caption>
        <thead className="bg-zinc-950 text-left text-xs uppercase tracking-[0.2em] text-zinc-500">
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
              Actions
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-white/8">
          {files.map((file) => (
            <tr key={`${file.job_id}:${file.path}`} className="hover:bg-zinc-950">
              <td className="px-4 py-3">
                <a
                  href={`/packages/view/?name=${encodeURIComponent(file.package_name)}`}
                  className="text-white transition hover:text-zinc-300"
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
              <td className="px-4 py-3 text-sm uppercase tracking-[0.18em] text-zinc-500">
                {file.kind}
              </td>
              <td className="px-4 py-3 text-sm text-zinc-400">
                {formatBytes(file.size_bytes)}
              </td>
              <td className="px-4 py-3 text-sm text-zinc-400">
                {formatDateTime(file.published_at)}
              </td>
              <td className="px-4 py-3">
                <div className="flex flex-wrap gap-2">
                  <a
                    href={`/repo/${file.path}`}
                    className="inline-flex items-center border border-zinc-800 px-3 py-2 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
                  >
                    <FaIcon icon={faDownload} className="mr-2" />
                    Download
                  </a>
                  <a
                    href={`/jobs/view/?id=${encodeURIComponent(file.job_id)}`}
                    className="inline-flex items-center border border-zinc-800 px-3 py-2 text-sm text-zinc-400 transition hover:border-zinc-600 hover:bg-zinc-950 hover:text-zinc-200"
                  >
                    Build
                  </a>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
