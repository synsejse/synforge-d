import { faTrash } from "@fortawesome/free-solid-svg-icons";
import FilterBar from "../../../components/common/filter-bar";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import Select from "../../../components/ui/select";
import {
  HISTORY_BUILD_STATUS_LABELS,
  isHistoryBuildStatus,
  type HistoryBuildStatus,
} from "../../../lib/job-status";
import type { JobViewMode } from "../types";

interface Props {
  mode: JobViewMode;
  status: "all" | HistoryBuildStatus;
  packageValue: string;
  targetValue: string;
  includeDeleted: boolean;
  activeCount: number;
  pruning: boolean;
  onPackageChange: (value: string) => void;
  onTargetChange: (value: string) => void;
  onStatusChange: (value: "all" | HistoryBuildStatus) => void;
  onIncludeDeletedChange: (value: boolean) => void;
  onClear: () => void;
  onPrune: () => void;
}

export default function JobListFilters({
  mode,
  status,
  packageValue,
  targetValue,
  includeDeleted,
  activeCount,
  pruning,
  onPackageChange,
  onTargetChange,
  onStatusChange,
  onIncludeDeletedChange,
  onClear,
  onPrune,
}: Props) {
  return (
    <FilterBar
      activeCount={activeCount}
      onClear={onClear}
      trailing={
        mode === "history" ? (
          <Button
            variant="danger"
            size="sm"
            onClick={onPrune}
            loading={pruning}
          >
            <FaIcon icon={faTrash} />
            Prune failed
          </Button>
        ) : null
      }
    >
      <div className="grid items-end gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <TextFilter
          label="Package"
          value={packageValue}
          onChange={onPackageChange}
          placeholder="Filter by package ..."
        />
        <TextFilter
          label="Target"
          value={targetValue}
          onChange={onTargetChange}
          placeholder="Filter by target ..."
        />
        {mode === "history" ? (
          <label className="block">
            <span className="block font-mono text-xs font-semibold uppercase tracking-[0.22em] text-soft">
              Status
            </span>
            <div className="mt-2.5">
              <Select
                options={[
                  { value: "all", label: "All statuses" },
                  ...Object.entries(HISTORY_BUILD_STATUS_LABELS).map(
                    ([value, label]) => ({ value, label }),
                  ),
                ]}
                value={status}
                onValueChange={(value) =>
                  onStatusChange(
                    isHistoryBuildStatus(value) ? value : "all",
                  )
                }
                placeholder="Filter status..."
              />
            </div>
          </label>
        ) : null}
        {mode === "history" ? (
          <label className="flex min-h-11 cursor-pointer items-center gap-3 border border-edge bg-black px-3 py-2 font-mono text-xs font-semibold uppercase tracking-[0.06em] text-soft transition-colors hover:text-strong">
            <input
              type="checkbox"
              checked={includeDeleted}
              onChange={(event) =>
                onIncludeDeletedChange(event.target.checked)
              }
            />
            Show deleted
          </label>
        ) : null}
      </div>
    </FilterBar>
  );
}

function TextFilter({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
}) {
  return (
    <label className="block">
      <span className="block font-mono text-xs font-semibold uppercase tracking-[0.22em] text-soft">
        {label}
      </span>
      <input
        type="text"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        className="mt-2.5 w-full border border-edge bg-black px-3 py-2.5 font-mono text-xs text-white outline-none transition-colors focus:border-accent-lime"
      />
    </label>
  );
}
