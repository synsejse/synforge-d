import Button from "../../../components/ui/button";
import type { Step } from "../model";

interface SetupNavProps {
  step: Step;
  submitting: boolean;
  onBack: () => void;
  onNext: () => void;
}

export default function SetupNav({
  step,
  submitting,
  onBack,
  onNext,
}: SetupNavProps) {
  return (
    <div className="mt-4 flex items-center justify-between gap-3">
      {step !== "config" ? (
        <Button type="button" variant="ghost" size="md" onClick={onBack}>
          Back
        </Button>
      ) : null}
      <div className="ml-auto flex items-center gap-3">
        {step !== "admin" ? (
          <Button type="button" variant="primary" size="md" onClick={onNext}>
            Continue
          </Button>
        ) : (
          <Button type="submit" variant="primary" size="md" loading={submitting}>
            {submitting ? "Initializing…" : "Initialize Synforge"}
          </Button>
        )}
      </div>
    </div>
  );
}
