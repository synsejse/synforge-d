import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";
import type { FormEvent } from "react";
import FaIcon from "../ui/FaIcon";

export type RepoKindFilter = "all" | "rpm" | "srpm" | "log";

interface RepositoryInventoryFiltersProps {
  packageFilter: string;
  targetFilter: string;
  kindFilter: RepoKindFilter;
  onPackageFilterChange: (value: string) => void;
  onTargetFilterChange: (value: string) => void;
  onKindFilterChange: (value: RepoKindFilter) => void;
  onApply: (event: FormEvent) => void;
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
      className="grid gap-3 border border-zinc-800 bg-black p-4 md:grid-cols-[minmax(0,1fr)_240px_180px_auto]"
    >
      <label className="block">
        <span className="sr-only">Filter by package name</span>
        <input
          type="search"
          value={packageFilter}
          onChange={(event) => onPackageFilterChange(event.target.value)}
          placeholder="Filter by package"
          className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
        />
      </label>
      <label className="block">
        <span className="sr-only">Filter by target</span>
        <input
          type="search"
          value={targetFilter}
          onChange={(event) => onTargetFilterChange(event.target.value)}
          placeholder="Filter by target"
          className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
        />
      </label>
      <label className="block">
        <span className="sr-only">Filter by artifact kind</span>
        <select
          value={kindFilter}
          onChange={(event) => onKindFilterChange(event.target.value as RepoKindFilter)}
          className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
        >
          <option value="all">All kinds</option>
          <option value="rpm">RPM</option>
          <option value="srpm">SRPM</option>
          <option value="log">Log</option>
        </select>
      </label>
      <button
        type="submit"
        className="border border-zinc-800 bg-black px-4 py-2.5 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
      >
        <FaIcon icon={faMagnifyingGlass} className="mr-2" />
        Apply
      </button>
    </form>
  );
}
