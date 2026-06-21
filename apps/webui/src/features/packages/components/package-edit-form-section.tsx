import { faMagnifyingGlass, faSave } from "@fortawesome/free-solid-svg-icons";
import { formatMockChroots } from "../../../lib/utils";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import {
  TextField,
  ToggleField,
  FieldGroup,
  DisplayBox,
} from "../../../components/ui/form-fields";
import PackageEditAdvancedSection from "./package-edit-advanced-section";
import { ChrootPickerDialog, SpecPickerDialog } from "./package-edit-pickers";
import type { PackageEditFormState } from "./package-edit-form-state";

export type { PackageEditFormState } from "./package-edit-form-state";

interface PackageEditFormSectionProps {
  form: PackageEditFormState;
  /** Pristine form state (what the package currently looks like in the API). */
  pristine: PackageEditFormState | null;
  maxCpuCores: number | null;
  maxMemoryMb: number | null;
  saving: boolean;
  availableChroots: string[];
  showSpecPicker: boolean;
  showChrootPicker: boolean;
  browsing: boolean;
  browseError: string | null;
  selectableFiles: string[];
  /** Persist the current form. Called from both the <form>'s native submit
   *  and the sticky footer's Save button. */
  onSubmit: () => void;
  onFormChange: (next: Partial<PackageEditFormState>) => void;
  onToggleChroot: (chroot: string, checked: boolean) => void;
  onOpenSpecPicker: () => void;
  onCloseSpecPicker: () => void;
  onOpenChrootPicker: () => void;
  onCloseChrootPicker: () => void;
  onBrowseRepository: () => void;
  /** Reset form back to pristine. Wired by the sticky footer's Discard button. */
  onDiscard: () => void;
}

export default function PackageEditFormSection({
  form,
  pristine,
  maxCpuCores,
  maxMemoryMb,
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
  onDiscard,
}: PackageEditFormSectionProps) {
  const isDirty =
    pristine != null && JSON.stringify(form) !== JSON.stringify(pristine);

  return (
    <>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          onSubmit();
        }}
        className="min-w-0 space-y-4"
      >
        {/* Always-visible essentials — Git URL, spec, chroots, enabled.
            Most package edits touch these and these only. */}
        <section className="space-y-5 border border-edge bg-black p-4 sm:p-6">
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
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={onOpenSpecPicker}
              >
                <FaIcon icon={faMagnifyingGlass} />
                Browse repository
              </Button>
            }
          >
            <input
              type="text"
              value={form.specPath}
              onChange={(event) => onFormChange({ specPath: event.target.value })}
              placeholder="path/to/package.spec"
              className="w-full border border-edge bg-black px-4 py-3 font-mono text-sm text-white placeholder:text-soft outline-none transition duration-100 ease-linear focus:border-accent-lime"
              required
            />
          </FieldGroup>

          <FieldGroup
            label="Mock chroots"
            description="Each selected chroot becomes a separate build job."
            action={
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={onOpenChrootPicker}
              >
                Choose chroots
              </Button>
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
        </section>

        {/* Advanced settings — polling/publish, resource limits, schedule. */}
        <PackageEditAdvancedSection
          form={form}
          maxCpuCores={maxCpuCores}
          maxMemoryMb={maxMemoryMb}
          onFormChange={onFormChange}
        />
      </form>

      {/* Sticky save bar — shown only while the form is dirty. */}
      {isDirty ? (
        <div
          role="region"
          aria-label="Unsaved changes"
          className="sticky bottom-0 z-30 -mx-3 mt-4 border-t border-accent-lime bg-black/95 px-4 py-3 backdrop-blur-sm sm:-mx-5 lg:-mx-8"
        >
          <div className="mx-auto flex max-w-[96rem] flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex items-center gap-3">
              <span
                aria-hidden="true"
                className="inline-block h-2 w-2 animate-pulse bg-accent-lime"
              />
              <span className="font-mono text-xs font-bold uppercase tracking-[0.18em] text-accent-lime">
                Unsaved changes
              </span>
            </div>
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-end">
              <Button
                variant="ghost"
                size="sm"
                fullWidth="responsive"
                onClick={onDiscard}
                disabled={saving}
              >
                Discard
              </Button>
              <Button
                variant="primary"
                size="sm"
                fullWidth="responsive"
                onClick={() => onSubmit()}
                loading={saving}
              >
                {saving ? null : <FaIcon icon={faSave} />}
                {saving ? "Saving…" : "Save changes"}
              </Button>
            </div>
          </div>
        </div>
      ) : null}

      {showSpecPicker && (
        <SpecPickerDialog
          specPath={form.specPath}
          selectableFiles={selectableFiles}
          browsing={browsing}
          browseError={browseError}
          onBrowseRepository={onBrowseRepository}
          onSelectSpec={(file) => {
            onFormChange({ specPath: file });
            onCloseSpecPicker();
          }}
          onClose={onCloseSpecPicker}
        />
      )}

      {showChrootPicker && (
        <ChrootPickerDialog
          availableChroots={availableChroots}
          selectedChroots={form.mockChroots}
          onToggleChroot={onToggleChroot}
          onClose={onCloseChrootPicker}
        />
      )}
    </>
  );
}
