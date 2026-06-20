import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import SelectionDialog from "../../../components/common/selection-dialog";

interface SpecPickerProps {
  specPath: string;
  selectableFiles: string[];
  browsing: boolean;
  browseError: string | null;
  onBrowseRepository: () => void;
  onSelectSpec: (file: string) => void;
  onClose: () => void;
}

export function SpecPickerDialog({
  specPath,
  selectableFiles,
  browsing,
  browseError,
  onBrowseRepository,
  onSelectSpec,
  onClose,
}: SpecPickerProps) {
  return (
    <SelectionDialog
      title="Choose spec file"
      subtitle="Browse the tracked repository and select the .spec file to build."
      onClose={onClose}
    >
      <div className="space-y-4">
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={onBrowseRepository}
          loading={browsing}
          disabled={browsing}
        >
          {browsing ? null : <FaIcon icon={faMagnifyingGlass} />}
          {browsing ? "Browsing…" : "Load repository files"}
        </Button>
        {browseError ? (
          <div className="border border-edge bg-black px-4 py-3 text-sm text-strong">
            {browseError}
          </div>
        ) : null}
        <div className="max-h-[50vh] overflow-auto border border-edge bg-black">
          {selectableFiles.length > 0 ? (
            selectableFiles.map((file) => (
              <button
                key={file}
                type="button"
                onClick={() => onSelectSpec(file)}
                className={`block w-full break-all border-b border-edge px-4 py-2 text-left font-mono text-sm transition last:border-b-0 ${
                  specPath === file
                    ? "bg-surface-alt text-white"
                    : "bg-black text-muted hover:bg-surface-alt"
                }`}
              >
                {file}
              </button>
            ))
          ) : (
            <div className="px-4 py-3 text-sm text-muted">
              No spec files loaded yet.
            </div>
          )}
        </div>
      </div>
    </SelectionDialog>
  );
}

interface ChrootPickerProps {
  availableChroots: string[];
  selectedChroots: string[];
  onToggleChroot: (chroot: string, checked: boolean) => void;
  onClose: () => void;
}

export function ChrootPickerDialog({
  availableChroots,
  selectedChroots,
  onToggleChroot,
  onClose,
}: ChrootPickerProps) {
  return (
    <SelectionDialog
      title="Choose mock chroots"
      subtitle="Select one or more build targets."
      onClose={onClose}
    >
      <div className="max-h-[50vh] overflow-y-auto border border-edge bg-black">
        <div className="divide-y divide-edge">
          {availableChroots.map((chroot) => (
            <label
              key={chroot}
              className="flex items-center justify-between gap-4 px-4 py-3 text-sm text-strong hover:bg-surface-alt"
            >
              <span className="font-mono">{chroot}</span>
              <input
                type="checkbox"
                checked={selectedChroots.includes(chroot)}
                onChange={(event) => onToggleChroot(chroot, event.target.checked)}
              />
            </label>
          ))}
        </div>
      </div>
    </SelectionDialog>
  );
}
