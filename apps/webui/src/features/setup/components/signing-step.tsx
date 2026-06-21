import type { RefObject } from "react";
import Button from "../../../components/ui/button";
import type { SigningState } from "../model";

interface SigningStepProps {
  signing: SigningState;
  onToggle: () => void;
  onSelectGenerate: () => void;
  onSelectImport: () => void;
  fileInputRef: RefObject<HTMLInputElement | null>;
  onFileChange: (file: File) => void;
}

export default function SigningStep({
  signing,
  onToggle,
  onSelectGenerate,
  onSelectImport,
  fileInputRef,
  onFileChange,
}: SigningStepProps) {
  const keyNote = !signing.enabled
    ? "Key actions are disabled while signing is disabled."
    : signing.mode === "import"
      ? signing.filename
        ? `Import selected: ${signing.filename}`
        : "Import mode selected. Choose a private key file."
      : "Managed key generation is selected.";

  return (
    <div className="space-y-4">
      <section className="border border-edge bg-black p-5">
        <h2 className="font-mono text-[13px] font-bold uppercase tracking-[0.06em] text-white">Signing</h2>
        <p className="mt-3 text-sm leading-6 text-muted">
          Configure repository signing before initialization. You can keep signing
          disabled, generate a managed key, or import an existing key file.
        </p>

        <div className="mt-5 space-y-5">
          <div>
            <p className="mb-2 font-mono text-xs font-bold uppercase tracking-[0.16em] text-muted">
              Signing State
            </p>
            <Button
              type="button"
              variant={signing.enabled ? "primary" : "ghost"}
              size="md"
              onClick={onToggle}
            >
              {signing.enabled ? "Disable Signing" : "Enable Signing"}
            </Button>
            <p className="mt-2 font-mono text-xs text-soft">
              {signing.enabled
                ? "Signing will be enabled after initialization."
                : "Signing will stay disabled after initialization."}
            </p>
          </div>

          <div>
            <p className="mb-2 font-mono text-xs font-bold uppercase tracking-[0.16em] text-muted">
              Key Lifecycle
            </p>
            <div className="flex flex-wrap gap-3">
              <Button
                type="button"
                variant={
                  signing.enabled && signing.mode === "generate"
                    ? "primary"
                    : "ghost"
                }
                size="md"
                disabled={!signing.enabled}
                onClick={onSelectGenerate}
              >
                Generate Key
              </Button>
              <Button
                type="button"
                variant={
                  signing.enabled && signing.mode === "import"
                    ? "primary"
                    : "ghost"
                }
                size="md"
                disabled={!signing.enabled}
                onClick={onSelectImport}
              >
                Import Key File
              </Button>
            </div>
            <input
              ref={fileInputRef}
              type="file"
              accept=".asc,.key,.pgp,.gpg,text/plain"
              className="hidden"
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (file) onFileChange(file);
              }}
            />
            <p className="mt-2 font-mono text-xs text-soft">{keyNote}</p>
          </div>
        </div>
      </section>
    </div>
  );
}
