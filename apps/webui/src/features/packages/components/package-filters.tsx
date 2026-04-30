import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";
import FaIcon from "../../../components/ui/fa-icon";

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
    <section className="grid gap-3 border-2 border-zinc-700 bg-black p-4 md:grid-cols-2 xl:grid-cols-[minmax(0,1fr)_220px_auto]">
      <label className="block">
        <span className="sr-only">Search packages</span>
        <input
          type="search"
          value={search}
          onChange={(event) => onSearchChange(event.target.value)}
          placeholder="Search name or description"
          className="w-full border-2 border-zinc-700 bg-black px-4 py-2.5 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]"
        />
      </label>
      <label className="block">
        <span className="sr-only">Filter by enabled state</span>
        <select
          value={enabledFilter}
          onChange={(event) =>
            onEnabledFilterChange(event.target.value as "all" | "true" | "false")
          }
          className="w-full border-2 border-zinc-700 bg-black px-4 py-2.5 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]"
        >
          <option value="all">All states</option>
          <option value="true">Enabled only</option>
          <option value="false">Disabled only</option>
        </select>
      </label>
      <button
        type="button"
        onClick={onApply}
        className="w-full border-2 border-zinc-700 bg-black px-4 py-2.5 font-mono text-xs font-bold uppercase tracking-[0.12em] text-zinc-200 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950 md:col-span-2 xl:col-span-1 xl:w-auto"
      >
        <FaIcon icon={faMagnifyingGlass} className="mr-2" />
        Apply
      </button>
    </section>
  );
}
