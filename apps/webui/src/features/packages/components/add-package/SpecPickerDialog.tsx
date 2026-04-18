import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";
import type { BrowseRepositoryProgressView } from "../../../../lib/types";
import SelectionDialog from "../../../../components/common/SelectionDialog";
import FaIcon from "../../../../components/ui/FaIcon";

interface SpecPickerDialogProps {
  activeBrowseProgress: BrowseRepositoryProgressView | null;
  browseError: string | null;
  browseProgressIssue: string | null;
  browseProgressMessage: string;
  browseProgressPercent: number;
  browseProgressState: BrowseRepositoryProgressView["state"];
  browsing: boolean;
  onBrowse: () => void;
  onClose: () => void;
  onSelectSpec: (file: string) => void;
  selectableFiles: string[];
  specPath: string;
}

export default function SpecPickerDialog({
  activeBrowseProgress,
  browseError,
  browseProgressIssue,
  browseProgressMessage,
  browseProgressPercent,
  browseProgressState,
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
          {browsing ? "Cloning repository…" : "Load repository files"}
        </button>
        {(browsing || activeBrowseProgress) && (
          <div className="border-2 border-zinc-700 bg-zinc-950 px-4 py-3">
            <div className="flex items-center justify-between gap-3">
              <span className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300">
                Git clone progress
              </span>
              <span className="font-mono text-xs text-zinc-300">
                {Math.round(browseProgressPercent)}%
              </span>
            </div>
            <div className="mt-3 h-2 w-full overflow-hidden border border-zinc-700 bg-zinc-900">
              <div
                className={`h-full transition-[width] duration-300 ${
                  browseProgressState === "failed"
                    ? "bg-red-500"
                    : browseProgressState === "completed"
                      ? "bg-[var(--theme-terminal-green)]"
                      : "bg-[var(--theme-accent-lime)]"
                }`}
                style={{
                  width: `${Math.max(0, Math.min(100, browseProgressPercent))}%`,
                }}
              />
            </div>
            <p className="mt-2 font-mono text-xs uppercase tracking-[0.12em] text-zinc-400">
              {stateLabel(browseProgressState)} · {browseProgressMessage}
            </p>
            {browseProgressIssue ? (
              <p className="mt-2 text-xs text-zinc-500">{browseProgressIssue}</p>
            ) : null}
          </div>
        )}
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

function stateLabel(state: BrowseRepositoryProgressView["state"]): string {
  switch (state) {
    case "completed":
      return "Completed";
    case "failed":
      return "Failed";
    default:
      return "Cloning";
  }
}
