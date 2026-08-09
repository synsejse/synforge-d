import Button from "../../../components/ui/button";
import EmptyState from "../../../components/ui/empty-state";

interface Props {
  filtered: boolean;
  onClearFilters: () => void;
  onAddPackage: () => void;
}

export default function PackageListEmptyState({
  filtered,
  onClearFilters,
  onAddPackage,
}: Props) {
  return (
    <EmptyState
      title={filtered ? "No matching packages" : "No packages configured"}
      description={
        filtered
          ? "Try a different search or clear the active filters."
          : "Add a spec source to start building."
      }
      action={
        <Button
          variant={filtered ? "subtle" : "primary"}
          onClick={filtered ? onClearFilters : onAddPackage}
        >
          {filtered ? "Clear filters" : "Add package"}
        </Button>
      }
    />
  );
}
