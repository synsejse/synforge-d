import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { useEffect, useId, useRef, type ReactNode } from "react";
import FaIcon from "../ui/FaIcon";

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
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 px-4 py-8"
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
        className="w-full max-w-xl border border-zinc-800 bg-black p-6 shadow-2xl"
      >
        <div className="mb-5 flex items-start justify-between gap-4">
          <div>
            <div className="text-xs uppercase tracking-[0.22em] text-zinc-500">
              Users
            </div>
            <h2 id={titleId} className="mt-2 text-2xl font-semibold text-white">
              {title}
            </h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="border border-zinc-800 px-3 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950"
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
        className="border border-zinc-800 bg-black px-4 py-2 text-sm font-medium text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
      >
        Cancel
      </button>
      <button
        type="submit"
        disabled={submitting}
        className="border border-zinc-200 bg-zinc-100 px-4 py-2 text-sm font-semibold text-black transition hover:bg-white disabled:opacity-70"
      >
        <FaIcon icon={submitIcon} className="mr-2" />
        {submitting ? "Saving…" : submitLabel}
      </button>
    </div>
  );
}
