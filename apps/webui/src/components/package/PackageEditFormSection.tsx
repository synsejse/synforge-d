import { faMagnifyingGlass, faSave } from "@fortawesome/free-solid-svg-icons";
import type { SyntheticEvent } from "react";
import { formatMockChroots } from "../../lib/utils";
import FaIcon from "../ui/FaIcon";
import SelectionDialog from "../common/SelectionDialog";
import {
  TextField,
  NumberField,
  TextAreaField,
  ToggleField,
  FieldGroup,
  DisplayBox,
} from "../ui/FormFields";

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
  onSubmit: (event: SyntheticEvent<HTMLFormElement>) => void;
  onFormChange: (next: Partial<PackageEditFormState>) => void;
  onToggleChroot: (chroot: string, checked: boolean) => void;
  onOpenSpecPicker: () => void;
  onCloseSpecPicker: () => void;
  onOpenChrootPicker: () => void;
  onCloseChrootPicker: () => void;
  onBrowseRepository: () => void;
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
      <form onSubmit={onSubmit} className="min-w-0 border-4 border-white bg-black p-4 sm:p-6">
        <div className="mb-6">
          <h2 className="font-mono text-xl font-bold uppercase text-white">Edit Package</h2>
          <p className="mt-2 text-sm text-zinc-400">
            Update the tracked repository, selected spec path, polling behavior,
            and package state from one place.
          </p>
        </div>

        <div className="space-y-5">
          <TextField
            label="Git repository URL"
            value={form.repoUrl}
            onChange={(value) => onFormChange({ repoUrl: value })}
            type="url"
            required
          />

          <FieldGroup
            label="Repository spec path"
            description="Choose the .spec file from the tracked repository."
            action={
              <button
                type="button"
                onClick={onOpenSpecPicker}
                className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
              >
                <FaIcon icon={faMagnifyingGlass} className="mr-2" />
                Browse repository
              </button>
            }
          >
            <input
              type="text"
              value={form.specPath}
              onChange={(event) => onFormChange({ specPath: event.target.value })}
              placeholder="path/to/package.spec"
              className="w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-white placeholder:text-zinc-600 outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)]"
              required
            />
          </FieldGroup>

          <div className="grid gap-4 md:grid-cols-2">
            <NumberField
              label="Poll interval (seconds)"
              value={form.pollIntervalSeconds}
              onChange={(value) => onFormChange({ pollIntervalSeconds: value })}
              required
            />

            <NumberField
              label="Build timeout (seconds)"
              value={form.buildTimeoutSeconds}
              onChange={(value) => onFormChange({ buildTimeoutSeconds: value })}
              required
            />

            <NumberField
              label="History count"
              value={form.packageHistoryCount}
              onChange={(value) => onFormChange({ packageHistoryCount: value })}
              required
              className="md:col-span-2"
            />

            <FieldGroup
              label="Mock chroots"
              description="Each selected chroot becomes a separate build job."
              className="md:col-span-2"
              action={
                <button
                  type="button"
                  onClick={onOpenChrootPicker}
                  className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
                >
                  Choose chroots
                </button>
              }
            >
              <DisplayBox>
                {form.mockChroots.length > 0
                  ? formatMockChroots(form.mockChroots, "No chroots selected")
                  : "No chroots selected"}
              </DisplayBox>
            </FieldGroup>

            <ToggleField
              label="Enabled"
              description="Allow new builds for this package."
              checked={form.enabled}
              onChange={(checked) => onFormChange({ enabled: checked })}
            />

            <ToggleField
              label="Source Polling"
              description="Watch the tracked git repository for new commits."
              checked={form.poll}
              onChange={(checked) => onFormChange({ poll: checked })}
            />

            <ToggleField
              label="Publish SRPM"
              description="Keep source RPM publication enabled for this package."
              checked={form.publish_srpm}
              onChange={(checked) => onFormChange({ publish_srpm: checked })}
            />

            <ToggleField
              label="Publish debug packages"
              description="Include debuginfo and debugsource RPMs in repository."
              checked={form.publish_debuginfo}
              onChange={(checked) => onFormChange({ publish_debuginfo: checked })}
            />

            <ToggleField
              label="Network access"
              description="Allow mock builds for this package to access the network."
              checked={form.network_access}
              onChange={(checked) => onFormChange({ network_access: checked })}
              className="md:col-span-2"
            />
          </div>

          <TextAreaField
            label="Build environment"
            value={form.buildEnv}
            onChange={(value) => onFormChange({ buildEnv: value })}
            placeholder="KEY=value&#10;MESON_ARGS=-Dgallium-drivers=swrast&#10;RUSTFLAGS=-C debuginfo=1"
            hint="One `KEY=value` entry per line. Applied to SRPM creation and mock rebuild steps."
          />

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
                    className={`block w-full break-all border-b-2 border-zinc-800 px-4 py-2 text-left font-mono text-sm transition last:border-b-0 ${
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
            <div className="divide-y divide-zinc-800">
              {availableChroots.map((chroot) => (
                <label
                  key={chroot}
                  className="flex items-center justify-between gap-4 px-4 py-3 text-sm text-zinc-200 hover:bg-zinc-950"
                >
                  <span className="font-mono">{chroot}</span>
                  <input
                    type="checkbox"
                    checked={form.mockChroots.includes(chroot)}
                    onChange={(event) =>
                      onToggleChroot(chroot, event.target.checked)
                    }
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
