import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";
import FaIcon from "../ui/FaIcon";

interface PackageFiltersProps {
  search: string;
  enabledFilter: "all" | "true" | "false";
  onSearchChange: (value: string) => void;
  onEnabledFilterChange: (value: "all" | "true" | "false") => void;
  onApply: () => void;
}

export default function PackageFilters({
  search,
  enabledFilter,
  onSearchChange,
  onEnabledFilterChange,
  onApply,
}: PackageFiltersProps) {
  return (
    <section className="grid gap-3 border border-zinc-800 bg-black p-4 md:grid-cols-[minmax(0,1fr)_220px_auto]">
      <label className="block">
        <span className="sr-only">Search packages</span>
        <input
          type="search"
          value={search}
          onChange={(event) => onSearchChange(event.target.value)}
          placeholder="Search name or description"
          className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
        />
      </label>
      <label className="block">
        <span className="sr-only">Filter by enabled state</span>
        <select
          value={enabledFilter}
          onChange={(event) =>
            onEnabledFilterChange(event.target.value as "all" | "true" | "false")
          }
          className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
        >
          <option value="all">All states</option>
          <option value="true">Enabled only</option>
          <option value="false">Disabled only</option>
        </select>
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
