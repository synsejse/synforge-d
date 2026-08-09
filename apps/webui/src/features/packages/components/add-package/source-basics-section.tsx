import { faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";
import Button from "../../../../components/ui/button";
import FaIcon from "../../../../components/ui/fa-icon";
import {
  FieldGroup,
  TextField,
} from "../../../../components/ui/form-fields";
import type { AddPackageFormState } from "./form-state";

interface Props {
  form: AddPackageFormState;
  browsing: boolean;
  onChange: (next: Partial<AddPackageFormState>) => void;
  onChooseSpec: () => void;
}

export default function SourceBasicsSection({
  form,
  browsing,
  onChange,
  onChooseSpec,
}: Props) {
  return (
    <div className="space-y-4">
      <div className="border-l-2 border-accent-lime bg-surface-alt px-4 py-3 text-sm text-muted">
        The package name is Synforge&apos;s identifier. The selected spec may
        produce RPMs with different names.
      </div>

      <TextField
        label="Package name"
        value={form.name}
        onChange={(name) => onChange({ name })}
        placeholder="mesa-git"
        required
      />

      <TextField
        label="Git repository URL"
        value={form.repoUrl}
        onChange={(repoUrl) => onChange({ repoUrl })}
        placeholder="https://github.com/example/repo.git or git@github.com:example/repo.git"
        hint="HTTP(S), SSH URLs, and git@host:path syntax are supported."
        required
      />

      <FieldGroup
        label="Repository spec path"
        description="Enter a path or browse the tracked repository for a .spec file."
        action={
          <Button
            variant="ghost"
            size="sm"
            onClick={onChooseSpec}
            disabled={browsing || form.repoUrl.trim().length === 0}
          >
            <FaIcon icon={faMagnifyingGlass} />
            Browse repository
          </Button>
        }
      >
        <input
          type="text"
          aria-label="Repository spec path"
          value={form.specPath}
          onChange={(event) => onChange({ specPath: event.target.value })}
          placeholder="path/to/package.spec"
          required
          className="w-full border border-edge bg-black px-4 py-3 font-mono text-sm text-white placeholder:text-soft outline-none transition duration-100 ease-linear focus:border-accent-lime"
        />
      </FieldGroup>
    </div>
  );
}
