import { faArrowLeft, faHammer, faRotate, faTrash } from "@fortawesome/free-solid-svg-icons";
import { Link } from "@tanstack/react-router";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";

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
    <section className="min-w-0 border-4 border-accent-lime bg-black p-4 sm:p-6">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <div className="min-w-0 flex-1 space-y-3">
          <Link
            to="/packages"
            className="inline-flex items-center font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted transition-all duration-100 ease-linear hover:text-strong"
          >
            <FaIcon icon={faArrowLeft} className="mr-2" />
            Back to packages
          </Link>
          <div>
            <p className="font-mono text-xs font-bold uppercase tracking-[0.3em] text-accent-lime">
              PACKAGE_CONTROL
            </p>
            <h1 className="mt-2 break-all font-mono text-2xl font-bold uppercase text-white sm:text-3xl">
              {packageName}
            </h1>
          </div>
          <p className="max-w-3xl text-sm text-muted">
            {description}
          </p>
        </div>
        <div className="grid gap-3 sm:flex sm:flex-wrap">
          <Button
            type="button"
            variant="secondary"
            size="md"
            fullWidth="responsive"
            iconLeft={faRotate}
            onClick={onRefresh}
            disabled={deleting || refreshing}
            loading={refreshing}
          >
            {refreshing ? "Refreshing…" : "Refresh"}
          </Button>
          <Button
            type="button"
            variant="primary"
            size="md"
            fullWidth="responsive"
            iconLeft={faHammer}
            onClick={onRebuild}
            disabled={deleting}
          >
            Rebuild
          </Button>
          <Button
            type="button"
            variant="danger"
            size="md"
            fullWidth="responsive"
            iconLeft={faTrash}
            onClick={onDelete}
            loading={deleting}
          >
            {deleting ? "Deleting…" : "Delete Package"}
          </Button>
        </div>
      </div>
    </section>
  );
}
