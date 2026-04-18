import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";
import type { SyntheticEvent } from "react";
import FaIcon from "../../../components/ui/FaIcon";

export type RepoKindFilter = "all" | "rpm" | "srpm" | "log";

interface RepositoryInventoryFiltersProps {
  packageFilter: string;
  targetFilter: string;
  kindFilter: RepoKindFilter;
  onPackageFilterChange: (value: string) => void;
  onTargetFilterChange: (value: string) => void;
  onKindFilterChange: (value: RepoKindFilter) => void;
  onApply: (event: SyntheticEvent) => void;
}

export default function RepositoryInventoryFilters({
  packageFilter,
  targetFilter,
  kindFilter,
  onPackageFilterChange,
  onTargetFilterChange,
  onKindFilterChange,
  onApply,
}: RepositoryInventoryFiltersProps) {
  return (
    <form
      onSubmit={onApply}
      className="grid gap-3 border-2 border-zinc-700 bg-black p-4 md:grid-cols-[minmax(0,1fr)_240px_180px_auto]"
    >
      <label className="block">
        <span className="sr-only">Filter by package name</span>
        <input
          type="search"
          value={packageFilter}
          onChange={(event) => onPackageFilterChange(event.target.value)}
          placeholder="Filter by package"
          className="w-full border-2 border-zinc-700 bg-black px-4 py-2.5 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]"
        />
      </label>
      <label className="block">
        <span className="sr-only">Filter by target</span>
        <input
          type="search"
          value={targetFilter}
          onChange={(event) => onTargetFilterChange(event.target.value)}
          placeholder="Filter by target"
          className="w-full border-2 border-zinc-700 bg-black px-4 py-2.5 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]"
        />
      </label>
      <label className="block">
        <span className="sr-only">Filter by artifact kind</span>
        <select
          value={kindFilter}
          onChange={(event) => onKindFilterChange(event.target.value as RepoKindFilter)}
          className="w-full border-2 border-zinc-700 bg-black px-4 py-2.5 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]"
        >
          <option value="all">All kinds</option>
          <option value="rpm">RPM</option>
          <option value="srpm">SRPM</option>
          <option value="log">Log</option>
        </select>
      </label>
      <button
        type="submit"
        className="border-2 border-zinc-700 bg-black px-4 py-2.5 font-mono text-xs font-bold uppercase tracking-[0.12em] text-zinc-200 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
      >
        <FaIcon icon={faMagnifyingGlass} className="mr-2" />
        Apply
      </button>
    </form>
  );
}
