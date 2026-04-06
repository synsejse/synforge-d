import { faMagnifyingGlass, faSave } from "@fortawesome/free-solid-svg-icons";
import type { FormEvent } from "react";
import FaIcon from "../ui/FaIcon";
import SelectionDialog from "../common/SelectionDialog";

export interface PackageEditFormState {
  repoUrl: string;
  specPath: string;
  poll: boolean;
  mockChroots: string[];
  pollIntervalSeconds: string;
  buildTimeoutSeconds: string;
  packageHistoryCount: string;
  buildEnv: string;
  enabled: boolean;
  publish_srpm: boolean;
  publish_debuginfo: boolean;
  network_access: boolean;
}

interface PackageEditFormSectionProps {
  form: PackageEditFormState;
  saving: boolean;
  availableChroots: string[];
  showSpecPicker: boolean;
  showChrootPicker: boolean;
  browsing: boolean;
  browseError: string | null;
  selectableFiles: string[];
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  onFormChange: (next: Partial<PackageEditFormState>) => void;
  onToggleChroot: (chroot: string, checked: boolean) => void;
  onOpenSpecPicker: () => void;
  onCloseSpecPicker: () => void;
  onOpenChrootPicker: () => void;
  onCloseChrootPicker: () => void;
  onBrowseRepository: () => void;
}

function formatMockChroots(chroots: string[]) {
  return chroots.join(", ");
}

export default function PackageEditFormSection({
  form,
  saving,
  availableChroots,
  showSpecPicker,
  showChrootPicker,
  browsing,
  browseError,
  selectableFiles,
  onSubmit,
  onFormChange,
  onToggleChroot,
  onOpenSpecPicker,
  onCloseSpecPicker,
  onOpenChrootPicker,
  onCloseChrootPicker,
  onBrowseRepository,
}: PackageEditFormSectionProps) {
  return (
    <>
      <form onSubmit={onSubmit} className="border-4 border-white bg-black p-6 shadow-[6px_6px_0_rgba(255,255,255,0.2)]">
        <div className="mb-6">
          <h2 className="font-mono text-xl font-bold uppercase text-white">Edit Package</h2>
          <p className="mt-2 text-sm text-zinc-400">
            Update the tracked repository, selected spec path, polling behavior,
            and package state from one place.
          </p>
        </div>

        <div className="space-y-5">
          <label className="block">
            <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.18em] text-zinc-400">
              Git repository URL
            </span>
            <input
              type="url"
              value={form.repoUrl}
              onChange={(event) => onFormChange({ repoUrl: event.target.value })}
              className="w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
              required
            />
          </label>

          <div className="border-2 border-zinc-700 bg-black p-4">
            <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
              <div>
                <span className="block text-sm font-medium text-zinc-300">
                  Repository spec path
                </span>
                <span className="mt-1 block text-xs text-zinc-500">
                  Choose the .spec file from the tracked repository.
                </span>
              </div>
              <button
                type="button"
                onClick={onOpenSpecPicker}
                className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
              >
                <FaIcon icon={faMagnifyingGlass} className="mr-2" />
                Browse repository
              </button>
            </div>
            <input
              type="text"
              value={form.specPath}
              onChange={(event) => onFormChange({ specPath: event.target.value })}
              placeholder="path/to/package.spec"
              className="mt-4 w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
              required
            />
          </div>

          <div className="grid gap-4 md:grid-cols-2">
            <label className="block">
              <span className="mb-2 block text-sm font-medium text-zinc-300">
                Poll interval (seconds)
              </span>
              <input
                type="number"
                min="1"
                step="1"
                value={form.pollIntervalSeconds}
                onChange={(event) =>
                  onFormChange({ pollIntervalSeconds: event.target.value })
                }
                className="w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
                required
              />
            </label>

            <label className="block">
              <span className="mb-2 block text-sm font-medium text-zinc-300">
                Build timeout (seconds)
              </span>
              <input
                type="number"
                min="1"
                step="1"
                value={form.buildTimeoutSeconds}
                onChange={(event) =>
                  onFormChange({ buildTimeoutSeconds: event.target.value })
                }
                className="w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
                required
              />
            </label>

            <label className="block md:col-span-2">
              <span className="mb-2 block text-sm font-medium text-zinc-300">
                History count
              </span>
              <input
                type="number"
                min="1"
                step="1"
                value={form.packageHistoryCount}
                onChange={(event) =>
                  onFormChange({ packageHistoryCount: event.target.value })
                }
                className="w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
                required
              />
            </label>

            <div className="border-2 border-zinc-700 bg-black p-4 md:col-span-2">
              <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <div>
                  <span className="block text-sm font-medium text-zinc-300">
                    Mock chroots
                  </span>
                  <span className="mt-1 block text-xs text-zinc-500">
                    Each selected chroot becomes a separate build job.
                  </span>
                </div>
                <button
                  type="button"
                  onClick={onOpenChrootPicker}
                  className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
                >
                  Choose chroots
                </button>
              </div>
              <div className="mt-4 border-2 border-zinc-700 bg-zinc-950 px-4 py-3 text-sm font-mono text-zinc-200">
                {form.mockChroots.length > 0
                  ? formatMockChroots(form.mockChroots)
                  : "No chroots selected"}
              </div>
            </div>

            <label className="flex items-center justify-between border-2 border-zinc-700 bg-black px-4 py-3">
              <span>
                <span className="block text-sm font-medium text-white">
                  Enabled
                </span>
                <span className="mt-1 block text-xs text-zinc-400">
                  Allow new builds for this package.
                </span>
              </span>
              <input
                type="checkbox"
                checked={form.enabled}
                onChange={(event) => onFormChange({ enabled: event.target.checked })}
                className="h-4 w-4 border-2 border-zinc-500 bg-black accent-[var(--theme-accent-lime)]"
              />
            </label>

            <label className="flex items-center justify-between border-2 border-zinc-700 bg-black px-4 py-3">
              <span>
                <span className="block text-sm font-medium text-white">
                  Source Polling
                </span>
                <span className="mt-1 block text-xs text-zinc-400">
                  Watch the tracked git repository for new commits.
                </span>
              </span>
              <input
                type="checkbox"
                checked={form.poll}
                onChange={(event) => onFormChange({ poll: event.target.checked })}
                className="h-4 w-4 border-2 border-zinc-500 bg-black accent-[var(--theme-accent-lime)]"
              />
            </label>

            <label className="flex items-center justify-between border-2 border-zinc-700 bg-black px-4 py-3">
              <span>
                <span className="block text-sm font-medium text-white">
                  Publish SRPM
                </span>
                <span className="mt-1 block text-xs text-zinc-400">
                  Keep source RPM publication enabled for this package.
                </span>
              </span>
              <input
                type="checkbox"
                checked={form.publish_srpm}
                onChange={(event) =>
                  onFormChange({ publish_srpm: event.target.checked })
                }
                className="h-4 w-4 border-2 border-zinc-500 bg-black accent-[var(--theme-accent-lime)]"
              />
            </label>

            <label className="flex items-center justify-between border-2 border-zinc-700 bg-black px-4 py-3">
              <span>
                <span className="block text-sm font-medium text-white">
                  Publish debug packages
                </span>
                <span className="mt-1 block text-xs text-zinc-400">
                  Include debuginfo and debugsource RPMs in repository.
                </span>
              </span>
              <input
                type="checkbox"
                checked={form.publish_debuginfo}
                onChange={(event) =>
                  onFormChange({ publish_debuginfo: event.target.checked })
                }
                className="h-4 w-4 border-2 border-zinc-500 bg-black accent-[var(--theme-accent-lime)]"
              />
            </label>

            <label className="flex items-center justify-between border-2 border-zinc-700 bg-black px-4 py-3 md:col-span-2">
              <span>
                <span className="block text-sm font-medium text-white">
                  Network access
                </span>
                <span className="mt-1 block text-xs text-zinc-400">
                  Allow mock builds for this package to access the network.
                </span>
              </span>
              <input
                type="checkbox"
                checked={form.network_access}
                onChange={(event) =>
                  onFormChange({ network_access: event.target.checked })
                }
                className="h-4 w-4 border-2 border-zinc-500 bg-black accent-[var(--theme-accent-lime)]"
              />
            </label>
          </div>

          <label className="block">
            <span className="mb-2 block text-sm font-medium text-zinc-300">
              Build environment
            </span>
            <textarea
              value={form.buildEnv}
              onChange={(event) => onFormChange({ buildEnv: event.target.value })}
              rows={6}
              placeholder={
                "KEY=value\nMESON_ARGS=-Dgallium-drivers=swrast\nRUSTFLAGS=-C debuginfo=1"
              }
              className="w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
            />
            <span className="mt-2 block text-xs text-zinc-500">
              One `KEY=value` entry per line. Applied to SRPM creation and mock
              rebuild steps.
            </span>
          </label>

          <div className="flex justify-end">
            <button
              type="submit"
              disabled={saving}
              className="border-2 border-[var(--theme-accent-lime)] bg-[var(--theme-accent-lime)] px-5 py-2.5 font-mono text-xs font-bold uppercase tracking-[0.15em] text-black transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:bg-[#d8ff72] disabled:opacity-70"
            >
              <FaIcon icon={faSave} className="mr-2" />
              {saving ? "Saving…" : "Save Changes"}
            </button>
          </div>
        </div>
      </form>

      {showSpecPicker && (
        <SelectionDialog
          title="Choose spec file"
          subtitle="Browse the tracked repository and select the .spec file to build."
          onClose={onCloseSpecPicker}
        >
          <div className="space-y-4">
            <button
              type="button"
              onClick={onBrowseRepository}
              disabled={browsing}
              className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950 disabled:opacity-60"
            >
              <FaIcon icon={faMagnifyingGlass} className="mr-2" />
              {browsing ? "Browsing…" : "Load repository files"}
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
                    onClick={() => {
                      onFormChange({ specPath: file });
                      onCloseSpecPicker();
                    }}
                    className={`block w-full border-b-2 border-zinc-800 px-4 py-2 text-left font-mono text-sm transition last:border-b-0 ${
                      form.specPath === file
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
      )}

      {showChrootPicker && (
        <SelectionDialog
          title="Choose mock chroots"
          subtitle="Select one or more build targets."
          onClose={onCloseChrootPicker}
        >
          <div className="max-h-[50vh] overflow-y-auto border-2 border-zinc-700 bg-black">
            <div className="divide-y divide-white/8">
              {availableChroots.map((chroot) => (
                <label
                  key={chroot}
                  className="flex items-center justify-between gap-4 px-4 py-3 text-sm text-zinc-200"
                >
                  <span className="font-mono">{chroot}</span>
                  <input
                    type="checkbox"
                    checked={form.mockChroots.includes(chroot)}
                    onChange={(event) =>
                      onToggleChroot(chroot, event.target.checked)
                    }
                    className="h-4 w-4 border-2 border-zinc-500 bg-black accent-[var(--theme-accent-lime)]"
                  />
                </label>
              ))}
            </div>
          </div>
        </SelectionDialog>
      )}
    </>
  );
}
