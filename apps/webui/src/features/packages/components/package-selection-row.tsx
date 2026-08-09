interface Props {
  packageNames: string[];
  selectedCount: number;
  allSelected: boolean;
  someSelected: boolean;
  onToggleAll: (selected: boolean) => void;
}

export default function PackageSelectionRow({
  packageNames,
  selectedCount,
  allSelected,
  someSelected,
  onToggleAll,
}: Props) {
  return (
    <div className="flex items-center justify-between gap-3 border border-edge bg-surface-alt px-4 py-3 font-mono text-xs uppercase tracking-[0.14em] text-soft">
      <label className="flex items-center gap-2.5 hover:text-white">
        <input
          type="checkbox"
          checked={allSelected}
          ref={(element) => {
            if (element) element.indeterminate = someSelected;
          }}
          onChange={(event) => onToggleAll(event.target.checked)}
          aria-label="Select all packages on this page"
        />
        Select all on page ({packageNames.length})
      </label>
      {selectedCount > 0 ? (
        <span className="text-soft">{selectedCount} total selected</span>
      ) : null}
    </div>
  );
}
