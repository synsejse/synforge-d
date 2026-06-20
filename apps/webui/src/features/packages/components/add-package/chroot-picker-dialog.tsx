import SelectionDialog from "../../../../components/common/selection-dialog";
import MockTargetCheckIndicator from "./mock-target-check-indicator";

interface ChrootPickerDialogProps {
  availableChroots: string[];
  chrootsLoading: boolean;
  mockChroots: string[];
  onClose: () => void;
  onToggleChroot: (chroot: string, checked: boolean) => void;
}

export default function ChrootPickerDialog({
  availableChroots,
  chrootsLoading,
  mockChroots,
  onClose,
  onToggleChroot,
}: ChrootPickerDialogProps) {
  return (
    <SelectionDialog
      title="Choose mock chroots"
      subtitle="Select one or more build targets."
      onClose={onClose}
    >
      <div className="max-h-[50vh] overflow-y-auto border border-edge bg-black">
        {chrootsLoading ? (
          <div className="px-4 py-3">
            <MockTargetCheckIndicator label="Checking mock targets…" />
          </div>
        ) : availableChroots.length === 0 ? (
          <div className="px-4 py-3 text-sm text-muted">
            No mock chroots available.
          </div>
        ) : (
          <div className="divide-y divide-edge">
            {availableChroots.map((chroot) => (
              <label
                key={chroot}
                className="flex items-center justify-between gap-4 px-4 py-3 text-sm text-strong"
              >
                <span className="font-mono">{chroot}</span>
                <input
                  type="checkbox"
                  checked={mockChroots.includes(chroot)}
                  onChange={(event) => onToggleChroot(chroot, event.target.checked)}
                  className="h-4 w-4 border-edge-strong bg-surface-hover"
                />
              </label>
            ))}
          </div>
        )}
      </div>
    </SelectionDialog>
  );
}
