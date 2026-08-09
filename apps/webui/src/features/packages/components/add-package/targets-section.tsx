import Button from "../../../../components/ui/button";
import {
  DisplayBox,
  FieldGroup,
  ToggleField,
} from "../../../../components/ui/form-fields";
import { formatMockChroots } from "../../../../lib/utils";
import type { AddPackageFormState } from "./form-state";
import MockTargetCheckIndicator from "./mock-target-check-indicator";

interface Props {
  form: AddPackageFormState;
  chrootsLoading: boolean;
  onChange: (next: Partial<AddPackageFormState>) => void;
  onChooseChroots: () => void;
}

export default function TargetsSection({
  form,
  chrootsLoading,
  onChange,
  onChooseChroots,
}: Props) {
  return (
    <div className="space-y-4">
      <FieldGroup
        label="Mock chroots"
        description="Each selected chroot becomes a separate build job."
        action={
          <Button
            variant="ghost"
            size="sm"
            onClick={onChooseChroots}
            disabled={chrootsLoading}
          >
            Choose chroots
          </Button>
        }
      >
        <DisplayBox>
          {chrootsLoading ? (
            <MockTargetCheckIndicator label="Checking mock targets…" />
          ) : (
            formatMockChroots(form.mockChroots, "No chroots selected")
          )}
        </DisplayBox>
      </FieldGroup>

      <div className="grid gap-4 md:grid-cols-2">
        <ToggleField
          label="Enabled"
          description="Allow new builds for this package."
          checked={form.enabled}
          onChange={(enabled) => onChange({ enabled })}
        />
        <ToggleField
          label="Enable polling"
          description="Automatically watch the source for updates."
          checked={form.poll}
          onChange={(poll) => onChange({ poll })}
        />
        <ToggleField
          label="Publish SRPM"
          description="Keep source RPMs in the repository."
          checked={form.publishSrpm}
          onChange={(publishSrpm) => onChange({ publishSrpm })}
        />
        <ToggleField
          label="Publish debug packages"
          description="Include debuginfo and debugsource RPMs."
          checked={form.publishDebuginfo}
          onChange={(publishDebuginfo) => onChange({ publishDebuginfo })}
        />
      </div>
    </div>
  );
}
