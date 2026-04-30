import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { useEffect, useId, useRef, type ReactNode } from "react";
import FaIcon from "../../../components/ui/fa-icon";

interface UserModalShellProps {
  title: string;
  children: ReactNode;
  onClose: () => void;
}

export function UserModalShell({ title, children, onClose }: UserModalShellProps) {
  const titleId = useId();
  const dialogRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const firstFocusable = dialogRef.current?.querySelector<HTMLElement>(
      'input, select, textarea, button, [href], [tabindex]:not([tabindex="-1"])',
    );
    firstFocusable?.focus();
  }, []);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 px-4 py-8"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="w-full max-w-xl border-4 border-white bg-black p-6 shadow-[6px_6px_0_rgba(255,255,255,0.25)]"
      >
        <div className="mb-5 flex items-start justify-between gap-4 border-b-2 border-zinc-800 pb-4">
          <div>
            <div className="font-mono text-xs font-bold uppercase tracking-[0.28em] text-[var(--theme-accent-lime)]">
              Users
            </div>
            <h2 id={titleId} className="mt-2 font-mono text-2xl font-bold uppercase text-white">
              {title}
            </h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="border-2 border-zinc-700 bg-black px-3 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-300 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
          >
            Close
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}

interface UserModalActionsProps {
  onClose: () => void;
  submitting: boolean;
  submitLabel: string;
  submitIcon: IconDefinition;
}

export function UserModalActions({
  onClose,
  submitting,
  submitLabel,
  submitIcon,
}: UserModalActionsProps) {
  return (
    <div className="flex justify-end gap-3">
      <button
        type="button"
        onClick={onClose}
        disabled={submitting}
        className="border-2 border-zinc-700 bg-black px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-zinc-200 transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-zinc-950"
      >
        Cancel
      </button>
      <button
        type="submit"
        disabled={submitting}
        className="border-2 border-[var(--theme-accent-lime)] bg-[var(--theme-accent-lime)] px-4 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-black transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:bg-[#d8ff72] disabled:opacity-70"
      >
        <FaIcon icon={submitIcon} className="mr-2" />
        {submitting ? "Saving…" : submitLabel}
      </button>
    </div>
  );
}
