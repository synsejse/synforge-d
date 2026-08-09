import { useEffect, useMemo, useState, type SyntheticEvent } from "react";
import {
  faArrowLeft,
  faArrowRight,
  faPlus,
  faXmark,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../../lib/api";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import ModalFrame from "../../../components/ui/modal-frame";
import { ModalTitle } from "../../../components/ui/modal-primitives";
import { useServerHardware } from "../../../components/common/server-hardware-provider";
import BuildSettingsSection from "./add-package/build-settings-section";
import ChrootPickerDialog from "./add-package/chroot-picker-dialog";
import {
  buildCreatePackageRequest,
  INITIAL_ADD_PACKAGE_FORM,
  type AddPackageFormState,
} from "./add-package/form-state";
import ReviewSection from "./add-package/review-section";
import SourceBasicsSection from "./add-package/source-basics-section";
import SpecPickerDialog from "./add-package/spec-picker-dialog";
import TargetsSection from "./add-package/targets-section";
import WizardSteps from "./add-package/wizard-steps";
import {
  ADD_PACKAGE_STEPS,
  type AddPackageStep,
} from "./add-package/wizard-step-data";

interface AddPackageModalProps {
  onClose: () => void;
  onSuccess: () => void;
}

const STEP_COPY: Record<AddPackageStep, { title: string; description: string }> = {
  source: {
    title: "Choose the source",
    description: "Name this package and select the spec tracked by its Git repository.",
  },
  targets: {
    title: "Choose build targets",
    description: "Select Mock chroots and decide what Synforge should publish.",
  },
  build: {
    title: "Tune the build",
    description: "Configure timeout and compiler caching; advanced limits are optional.",
  },
  review: {
    title: "Review package",
    description: "Confirm the source, targets, and build behavior before creating it.",
  },
};

export default function AddPackageModal({
  onClose,
  onSuccess,
}: AddPackageModalProps) {
  const serverHardware = useServerHardware();
  const [form, setForm] = useState<AddPackageFormState>(
    INITIAL_ADD_PACKAGE_FORM,
  );
  const [step, setStep] = useState<AddPackageStep>("source");
  const [browsing, setBrowsing] = useState(false);
  const [browseError, setBrowseError] = useState<string | null>(null);
  const [browseFiles, setBrowseFiles] = useState<string[]>([]);
  const [availableChroots, setAvailableChroots] = useState<string[]>([]);
  const [chrootsLoading, setChrootsLoading] = useState(true);
  const [showSpecPicker, setShowSpecPicker] = useState(false);
  const [showChrootPicker, setShowChrootPicker] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const selectableFiles = useMemo(
    () => browseFiles.filter((file) => file.endsWith(".spec")),
    [browseFiles],
  );
  const currentStepIndex = ADD_PACKAGE_STEPS.findIndex(
    (entry) => entry.value === step,
  );
  const patchForm = (next: Partial<AddPackageFormState>) =>
    setForm((current) => ({ ...current, ...next }));

  useEffect(() => {
    async function loadChroots() {
      try {
        const response = await api.listMockChroots();
        setAvailableChroots(response.chroots);
        setForm((current) => ({
          ...current,
          mockChroots: chooseInitialChroots(
            current.mockChroots,
            response.chroots,
          ),
        }));
      } catch (loadError) {
        setError(
          loadError instanceof Error
            ? loadError.message
            : "Failed to load mock chroots",
        );
      } finally {
        setChrootsLoading(false);
      }
    }

    void loadChroots();
  }, []);

  async function handleBrowse() {
    const trimmedRepoUrl = form.repoUrl.trim();
    if (!trimmedRepoUrl) {
      setBrowseError("Repository URL is required before browsing.");
      return;
    }
    setBrowsing(true);
    setBrowseError(null);
    try {
      const response = await api.browseRepository({ repo_url: trimmedRepoUrl });
      setBrowseFiles(response.files);
      if (!form.specPath && response.spec_files.length > 0) {
        patchForm({ specPath: response.spec_files[0] });
      }
    } catch (browseFailure) {
      setBrowseError(
        browseFailure instanceof Error
          ? browseFailure.message
          : "Failed to browse repository",
      );
    } finally {
      setBrowsing(false);
    }
  }

  function validate(nextStep: AddPackageStep): string | null {
    if (nextStep === "source") {
      if (!form.name.trim()) return "Package name is required.";
      if (!form.repoUrl.trim()) return "Git repository URL is required.";
      if (!form.specPath.trim()) return "Choose or enter a spec file path.";
    }
    if (nextStep === "targets" && form.mockChroots.length === 0) {
      return "Select at least one Mock chroot.";
    }
    if (nextStep === "build") {
      if (!isPositiveNumber(form.buildTimeoutSeconds)) {
        return "Build timeout must be greater than zero.";
      }
      if (form.poll && !isPositiveNumber(form.pollIntervalSeconds)) {
        return "Poll interval must be greater than zero.";
      }
      if (!isPositiveNumber(form.packageHistoryCount)) {
        return "History count must be greater than zero.";
      }
      if (
        form.ccacheEnabled &&
        form.ccacheMaxSizeMb.trim() &&
        !isPositiveNumber(form.ccacheMaxSizeMb)
      ) {
        return "Compiler-cache size must be greater than zero.";
      }
    }
    return null;
  }

  function goNext() {
    const validationError = validate(step);
    if (validationError) {
      setError(validationError);
      return;
    }
    const nextStep = ADD_PACKAGE_STEPS[currentStepIndex + 1];
    if (nextStep) {
      setStep(nextStep.value);
      setError(null);
    }
  }

  function goBack() {
    const previousStep = ADD_PACKAGE_STEPS[currentStepIndex - 1];
    if (previousStep) {
      setStep(previousStep.value);
      setError(null);
    }
  }

  async function handleSubmit(event: SyntheticEvent) {
    event.preventDefault();
    if (step !== "review") {
      goNext();
      return;
    }

    for (const entry of ADD_PACKAGE_STEPS.slice(0, -1)) {
      const validationError = validate(entry.value);
      if (validationError) {
        setStep(entry.value);
        setError(validationError);
        return;
      }
    }

    setSubmitting(true);
    setError(null);
    try {
      await api.createPackage(
        buildCreatePackageRequest(
          form,
          serverHardware?.cpu_cores ?? null,
        ),
      );
      onSuccess();
    } catch (submitError) {
      setError(
        submitError instanceof Error
          ? submitError.message
          : "Failed to create package",
      );
    } finally {
      setSubmitting(false);
    }
  }

  function toggleChroot(chroot: string, checked: boolean) {
    patchForm({
      mockChroots: checked
        ? Array.from(new Set([...form.mockChroots, chroot]))
        : form.mockChroots.filter((value) => value !== chroot),
    });
  }

  const copy = STEP_COPY[step];

  return (
    <ModalFrame
      open
      dismissable={!submitting}
      onOpenChange={(open) => {
        if (!open && !submitting) onClose();
      }}
      overlayClassName="flex items-center justify-center overflow-hidden overscroll-none bg-black/70 px-4 py-6"
      className="flex max-h-[calc(100dvh-3rem)] max-w-3xl flex-col overflow-hidden border border-edge-strong bg-black shadow-card-md"
    >
      <header className="flex shrink-0 items-center justify-between gap-4 border-b border-edge px-6 py-4">
        <div>
          <ModalTitle asChild>
            <h2 className="font-mono text-lg font-bold uppercase tracking-[0.04em] text-white">
              Add package
            </h2>
          </ModalTitle>
          <p className="mt-1 font-mono text-xs uppercase tracking-[0.12em] text-soft">
            Step {currentStepIndex + 1} of {ADD_PACKAGE_STEPS.length}
          </p>
        </div>
        <Button
          variant="ghost"
          size="icon"
          onClick={onClose}
          disabled={submitting}
          aria-label="Close add package dialog"
        >
          <FaIcon icon={faXmark} />
        </Button>
      </header>

      <WizardSteps current={step} />

      <form onSubmit={handleSubmit} className="flex min-h-0 flex-1 flex-col">
        <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain px-6 py-6">
          <div className="mb-5">
            <h3 className="font-display text-xl font-bold text-white">{copy.title}</h3>
            <p className="mt-1 text-sm text-muted">{copy.description}</p>
          </div>

          {step === "source" ? (
            <SourceBasicsSection
              form={form}
              browsing={browsing}
              onChange={patchForm}
              onChooseSpec={() => setShowSpecPicker(true)}
            />
          ) : null}
          {step === "targets" ? (
            <TargetsSection
              form={form}
              chrootsLoading={chrootsLoading}
              onChange={patchForm}
              onChooseChroots={() => setShowChrootPicker(true)}
            />
          ) : null}
          {step === "build" ? (
            <BuildSettingsSection
              form={form}
              maxCpuCores={serverHardware?.cpu_cores ?? null}
              maxMemoryMb={serverHardware?.total_memory_mb ?? null}
              onChange={patchForm}
            />
          ) : null}
          {step === "review" ? <ReviewSection form={form} /> : null}

          {error ? (
            <div role="alert" className="mt-5 border border-error bg-error/10 px-4 py-3 text-sm text-error">
              {error}
            </div>
          ) : null}
        </div>

        <footer className="flex shrink-0 flex-wrap items-center justify-between gap-3 border-t border-edge bg-black px-6 py-4">
          <Button variant="ghost" size="sm" onClick={onClose} disabled={submitting}>
            Cancel
          </Button>
          <div className="flex items-center gap-3">
            {currentStepIndex > 0 ? (
              <Button variant="subtle" size="sm" onClick={goBack} disabled={submitting}>
                <FaIcon icon={faArrowLeft} />
                Back
              </Button>
            ) : null}
            <Button type="submit" variant="primary" size="sm" loading={submitting}>
              {step === "review" ? (
                <>
                  {submitting ? null : <FaIcon icon={faPlus} />}
                  {submitting ? "Adding…" : "Add package"}
                </>
              ) : (
                <>
                  Continue
                  <FaIcon icon={faArrowRight} />
                </>
              )}
            </Button>
          </div>
        </footer>
      </form>

      {showSpecPicker ? (
        <SpecPickerDialog
          browseError={browseError}
          browsing={browsing}
          onBrowse={handleBrowse}
          onClose={() => setShowSpecPicker(false)}
          onSelectSpec={(file) => {
            patchForm({ specPath: file });
            setShowSpecPicker(false);
          }}
          selectableFiles={selectableFiles}
          specPath={form.specPath}
        />
      ) : null}

      {showChrootPicker ? (
        <ChrootPickerDialog
          availableChroots={availableChroots}
          chrootsLoading={chrootsLoading}
          mockChroots={form.mockChroots}
          onClose={() => setShowChrootPicker(false)}
          onToggleChroot={toggleChroot}
        />
      ) : null}
    </ModalFrame>
  );
}

function chooseInitialChroots(current: string[], available: string[]): string[] {
  const stillAvailable = current.filter((value) => available.includes(value));
  if (stillAvailable.length > 0) return stillAvailable;
  if (available.includes("fedora-44-x86_64")) return ["fedora-44-x86_64"];
  return available.length > 0 ? [available[0]] : [];
}

function isPositiveNumber(value: string): boolean {
  const number = Number(value);
  return Number.isFinite(number) && number > 0;
}
