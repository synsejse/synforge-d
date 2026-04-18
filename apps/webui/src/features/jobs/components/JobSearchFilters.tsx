import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";
import FaIcon from "../../../components/ui/FaIcon";

interface JobSearchFiltersProps {
  packageFilter: string;
  targetFilter: string;
  onPackageFilterChange: (value: string) => void;
  onTargetFilterChange: (value: string) => void;
  onApply: () => void;
}

export default function JobSearchFilters({
  packageFilter,
  targetFilter,
  onPackageFilterChange,
  onTargetFilterChange,
  onApply,
}: JobSearchFiltersProps) {
  return (
    <section className="grid gap-3 border border-zinc-800 bg-black p-4 md:grid-cols-[minmax(0,1fr)_240px_auto]">
      <label className="block">
        <span className="sr-only">Filter jobs by package</span>
        <input
          type="search"
          value={packageFilter}
          onChange={(event) => onPackageFilterChange(event.target.value)}
          placeholder="Filter by package"
          className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
        />
      </label>
      <label className="block">
        <span className="sr-only">Filter jobs by target</span>
        <input
          type="search"
          value={targetFilter}
          onChange={(event) => onTargetFilterChange(event.target.value)}
          placeholder="Filter by target"
          className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
        />
      </label>
      <button
        type="button"
        onClick={onApply}
        className="border border-zinc-800 bg-black px-4 py-2.5 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
      >
        <FaIcon icon={faMagnifyingGlass} className="mr-2" />
        Apply
      </button>
    </section>
  );
}
