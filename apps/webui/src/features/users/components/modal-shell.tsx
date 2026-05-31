import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import type { ReactNode } from "react";
import Button from "../../../components/ui/button";
import FaIcon from "../../../components/ui/fa-icon";
import ModalFrame, { ModalTitle } from "../../../components/ui/modal-frame";

interface UserModalShellProps {
  title: string;
  children: ReactNode;
  onClose: () => void;
}

export function UserModalShell({ title, children, onClose }: UserModalShellProps) {
  return (
    <ModalFrame
      open
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
      overlayClassName="flex items-center justify-center px-4 py-8"
      className="max-w-xl border-4 border-white bg-black p-6 shadow-card-md"
    >
      <div className="mb-5 border-b-2 border-edge pb-4">
        <ModalTitle asChild>
          <h2 className="text-2xl font-bold text-white">{title}</h2>
        </ModalTitle>
      </div>
      {children}
    </ModalFrame>
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
