import { faArrowLeft, faHammer, faRotate, faTrash } from "@fortawesome/free-solid-svg-icons";
import FaIcon from "./FaIcon";

interface PackageDetailHeaderProps {
  packageName: string;
  description: string;
  deleting: boolean;
  onRefresh: () => void;
  onRebuild: () => void;
  onDelete: () => void;
}

export default function PackageDetailHeader({
  packageName,
  description,
  deleting,
  onRefresh,
  onRebuild,
  onDelete,
}: PackageDetailHeaderProps) {
  return (
    <section className="border border-zinc-800 bg-black p-6">
      <div className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
        <div className="space-y-3">
          <a
            href="/packages/"
            className="text-sm text-zinc-400 transition hover:text-zinc-100"
          >
            <FaIcon icon={faArrowLeft} className="mr-2" />
            Back to packages
          </a>
          <div>
            <p className="text-xs uppercase tracking-[0.28em] text-zinc-500">
              Package Control
            </p>
            <h1 className="mt-2 text-4xl font-semibold tracking-tight text-white">
              {packageName}
            </h1>
          </div>
          <p className="max-w-3xl text-sm leading-6 text-zinc-300">
            {description}
          </p>
        </div>
        <div className="flex flex-wrap gap-3">
          <button
            onClick={onRefresh}
            className="border border-zinc-800 bg-black px-4 py-2 text-sm font-medium text-zinc-100 transition hover:border-zinc-600 hover:bg-zinc-950"
          >
            <FaIcon icon={faRotate} className="mr-2" />
            Refresh
          </button>
          <button
            onClick={onRebuild}
            className="border border-zinc-200 bg-zinc-100 px-4 py-2 text-sm font-semibold text-black transition hover:bg-white"
          >
            <FaIcon icon={faHammer} className="mr-2" />
            Rebuild
          </button>
          <button
            onClick={onDelete}
            disabled={deleting}
            className="border border-zinc-800 bg-black px-4 py-2 text-sm font-medium text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950 disabled:opacity-60"
          >
            <FaIcon icon={faTrash} className="mr-2" />
            {deleting ? "Deleting…" : "Delete Package"}
          </button>
        </div>
      </div>
    </section>
  );
}
