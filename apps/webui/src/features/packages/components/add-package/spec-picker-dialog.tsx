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
          className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950 disabled:opacity-60"
        >
          <FaIcon icon={faMagnifyingGlass} className="mr-2" />
          {browsing ? "Loading repository files…" : "Load repository files"}
        </button>
        {browseError ? (
          <div className="border-2 border-zinc-700 bg-black px-4 py-3 text-sm text-zinc-200">
            {browseError}
          </div>
        ) : null}
        <div className="max-h-[50vh] overflow-auto border-2 border-zinc-700 bg-black">
          {selectableFiles.length > 0 ? (
            selectableFiles.map((file) => (
              <button
                key={file}
                type="button"
                onClick={() => onSelectSpec(file)}
                className={`block w-full border-b-2 border-zinc-800 px-4 py-2 text-left font-mono text-sm transition last:border-b-0 ${
                  specPath === file
                    ? "bg-zinc-950 text-white"
                    : "bg-black text-zinc-300 hover:bg-zinc-950"
                }`}
              >
                {file}
              </button>
            ))
          ) : (
            <div className="px-4 py-3 text-sm text-zinc-400">
              No spec files loaded yet.
            </div>
          )}
        </div>
      </div>
    </SelectionDialog>
  );
}
