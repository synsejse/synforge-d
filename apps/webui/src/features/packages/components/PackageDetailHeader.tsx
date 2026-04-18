import { faArrowLeft, faHammer, faRotate, faTrash } from "@fortawesome/free-solid-svg-icons";
import Button from "../../../components/ui/Button";
import FaIcon from "../../../components/ui/FaIcon";

interface PackageDetailHeaderProps {
  packageName: string;
  description: string;
  deleting: boolean;
  refreshing: boolean;
  onRefresh: () => void;
  onRebuild: () => void;
  onDelete: () => void;
}

export default function PackageDetailHeader({
  packageName,
  description,
  deleting,
  refreshing,
  onRefresh,
  onRebuild,
  onDelete,
}: PackageDetailHeaderProps) {
  return (
    <section className="min-w-0 border-4 border-[var(--theme-accent-lime)] bg-black p-4 sm:p-6">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <div className="min-w-0 flex-1 space-y-3">
          <a
            href="/packages/"
            className="inline-flex items-center font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-400 transition-all duration-100 ease-linear hover:text-zinc-100"
          >
            <FaIcon icon={faArrowLeft} className="mr-2" />
            Back to packages
          </a>
          <div>
            <p className="font-mono text-xs font-bold uppercase tracking-[0.3em] text-[var(--theme-accent-lime)]">
              PACKAGE_CONTROL
            </p>
            <h1 className="mt-2 break-all font-mono text-2xl font-bold uppercase text-white sm:text-3xl">
              {packageName}
            </h1>
          </div>
          <p className="max-w-3xl text-sm text-zinc-400">
            {description}
          </p>
        </div>
        <div className="grid gap-3 sm:flex sm:flex-wrap">
          <Button
            type="button"
            variant="secondary"
            size="md"
            className="w-full sm:w-auto"
            onClick={onRefresh}
            disabled={deleting || refreshing}
            aria-busy={refreshing || undefined}
          >
            <FaIcon icon={faRotate} className={refreshing ? "mr-2 animate-spin" : "mr-2"} />
            {refreshing ? "Refreshing…" : "Refresh"}
          </Button>
          <button
            onClick={onRebuild}
            className="w-full border-2 border-[var(--theme-accent-lime)] bg-[var(--theme-accent-lime)] px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-black transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:bg-[#d8ff72] sm:w-auto"
          >
            <FaIcon icon={faHammer} className="mr-2" />
            Rebuild
          </button>
          <button
            onClick={onDelete}
            disabled={deleting}
            className="w-full border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950 disabled:opacity-60 sm:w-auto"
          >
            <FaIcon icon={faTrash} className="mr-2" />
            {deleting ? "Deleting…" : "Delete Package"}
          </button>
        </div>
      </div>
    </section>
  );
}
