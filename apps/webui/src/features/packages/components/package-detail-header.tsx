import { faHammer, faRotate, faTrash } from "@fortawesome/free-solid-svg-icons";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import { packageAccent } from "./package-accent";

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
    <header className="sticky -top-3 z-20 -mx-3 min-w-0 border-b-2 border-edge-strong bg-black/95 px-3 pb-4 pt-3 backdrop-blur-sm sm:-top-5 sm:-mx-5 sm:px-5 lg:-top-8 lg:-mx-8 lg:px-8">
      <div className="flex items-stretch gap-4">
        <span
          aria-hidden="true"
          className="shrink-0 w-1"
          style={{ background: packageAccent(packageName) }}
        />
        <div className="flex min-w-0 flex-1 flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div className="min-w-0 flex-1">
          <h1 className="break-all font-mono text-2xl font-bold uppercase text-white sm:text-3xl">
            {packageName}
          </h1>
          {description ? (
            <p className="mt-2 max-w-3xl text-sm text-muted">{description}</p>
          ) : null}
        </div>
        <div className="flex flex-wrap gap-2 lg:flex-nowrap">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            fullWidth="responsive-lg"
            onClick={onRefresh}
            disabled={deleting || refreshing}
            loading={refreshing}
          >
            {refreshing ? null : <FaIcon icon={faRotate} />}
            {refreshing ? "Refreshing…" : "Refresh"}
          </Button>
          <Button
            type="button"
            variant="primary"
            size="sm"
            fullWidth="responsive-lg"
            onClick={onRebuild}
            disabled={deleting}
          >
            <FaIcon icon={faHammer} />
            Rebuild
          </Button>
          <Button
            type="button"
            variant="danger"
            size="sm"
            fullWidth="responsive-lg"
            onClick={onDelete}
            loading={deleting}
          >
            {deleting ? null : <FaIcon icon={faTrash} />}
            {deleting ? "Deleting…" : "Delete"}
          </Button>
        </div>
      </div>
      </div>
    </header>
  );
}
