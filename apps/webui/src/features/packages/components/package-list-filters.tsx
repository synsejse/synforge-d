import Select from "../../../components/ui/select";
import FilterBar from "../../../components/common/filter-bar";

export type EnabledFilter = "all" | "true" | "false";

interface Props {
  search: string;
  enabled: EnabledFilter;
  onSearchChange: (search: string) => void;
  onEnabledChange: (enabled: EnabledFilter) => void;
  onClear: () => void;
}

export default function PackageListFilters({
  search,
  enabled,
  onSearchChange,
  onEnabledChange,
  onClear,
}: Props) {
  const activeCount = Number(search.trim().length > 0) + Number(enabled !== "all");

  return (
    <FilterBar activeCount={activeCount} onClear={onClear}>
      <div className="grid items-end gap-4 sm:grid-cols-[minmax(0,1fr)_200px]">
        <label className="block">
          <span className="block font-mono text-xs font-semibold uppercase tracking-[0.22em] text-soft">
            Search
          </span>
          <input
            type="text"
            value={search}
            onChange={(event) => onSearchChange(event.target.value)}
            placeholder="Filter by name or description"
            className="mt-2.5 w-full border border-edge bg-black px-3 py-2.5 font-mono text-[13px] text-white outline-none transition-colors focus:border-accent-lime"
          />
        </label>
        <label className="block">
          <span className="block font-mono text-xs font-semibold uppercase tracking-[0.22em] text-soft">
            Status
          </span>
          <div className="mt-2.5">
            <Select
              value={enabled}
              onValueChange={(value) => onEnabledChange(value as EnabledFilter)}
              options={[
                { value: "all", label: "All" },
                { value: "true", label: "Enabled" },
                { value: "false", label: "Disabled" },
              ]}
            />
          </div>
        </label>
      </div>
    </FilterBar>
  );
}
