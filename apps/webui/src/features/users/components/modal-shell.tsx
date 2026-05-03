import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { useEffect, useId, useRef, type ReactNode } from "react";
import Button from "../../../components/ui/button";
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
        <div className="mb-5 border-b-2 border-edge pb-4">
          <h2 id={titleId} className="text-2xl font-bold text-white">
            {title}
          </h2>
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
      <Button
        type="button"
        variant="ghost"
        size="sm"
        onClick={onClose}
        disabled={submitting}
      >
        Cancel
      </Button>
      <Button type="submit" variant="primary" size="sm" loading={submitting}>
        {submitting ? null : <FaIcon icon={submitIcon} />}
        {submitting ? "Saving…" : submitLabel}
      </Button>
    </div>
  );
}
