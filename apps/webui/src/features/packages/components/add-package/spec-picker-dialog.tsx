import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";
import SelectionDialog from "../../../../components/common/selection-dialog";
import FaIcon from "../../../../components/ui/fa-icon";

interface SpecPickerDialogProps {
  browseError: string | null;
  browsing: boolean;
  onBrowse: () => void;
  onClose: () => void;
  onSelectSpec: (file: string) => void;
  selectableFiles: string[];
  specPath: string;
}

export default function SpecPickerDialog({
  browseError,
  browsing,
  onBrowse,
  onClose,
  onSelectSpec,
  selectableFiles,
  specPath,
}: SpecPickerDialogProps) {
  return (
    <SelectionDialog
      title="Choose spec file"
      subtitle="Browse the repository and select the .spec file to build."
      onClose={onClose}
    >
      <div className="space-y-4">
        <button
          type="button"
          onClick={onBrowse}
          disabled={browsing}
          className="border border-edge bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted transition duration-100 ease-linear hover:border-white hover:bg-surface-alt disabled:opacity-60"
        >
          <FaIcon icon={faMagnifyingGlass} className="mr-2" />
          {browsing ? "Loading repository files…" : "Load repository files"}
        </button>
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
                className={`block w-full border-b border-edge px-4 py-2 text-left font-mono text-sm transition last:border-b-0 ${
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
