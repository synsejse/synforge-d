import {
  ADD_PACKAGE_STEPS,
  type AddPackageStep,
} from "./wizard-step-data";

export default function WizardSteps({ current }: { current: AddPackageStep }) {
  const currentIndex = ADD_PACKAGE_STEPS.findIndex(
    (step) => step.value === current,
  );

  return (
    <ol
      aria-label="Package creation progress"
      className="grid grid-cols-4 border-b border-edge bg-surface-alt"
    >
      {ADD_PACKAGE_STEPS.map((step, index) => {
        const active = index === currentIndex;
        const complete = index < currentIndex;
        return (
          <li
            key={step.value}
            aria-current={active ? "step" : undefined}
            className={`border-r border-edge px-2 py-3 text-center font-mono text-xs font-bold uppercase tracking-[0.08em] last:border-r-0 sm:px-4 ${
              active
                ? "bg-black text-accent-lime"
                : complete
                  ? "text-strong"
                  : "text-soft"
            }`}
          >
            <span className="hidden sm:inline">{index + 1}. </span>
            {step.label}
          </li>
        );
      })}
    </ol>
  );
}
