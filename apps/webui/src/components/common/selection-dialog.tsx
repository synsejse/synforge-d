import type { ReactNode } from "react";
import Button from "../ui/button";
import ModalFrame, { ModalDescription, ModalTitle } from "../ui/modal-frame";

interface SelectionDialogProps {
  title: string;
  subtitle: string;
  onClose: () => void;
  children: ReactNode;
}

export default function SelectionDialog({
  title,
  subtitle,
  onClose,
  children,
}: SelectionDialogProps) {
  return (
    <ModalFrame
      open
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
      zIndex={60}
      overlayClassName="overflow-y-auto px-4 py-8"
      className="top-8 max-h-[calc(100dvh-4rem)] max-w-3xl translate-y-0 overflow-y-auto border border-edge-strong bg-black shadow-card-md"
    >
      <div className="flex items-start justify-between gap-4 border-b-2 border-edge px-6 py-5">
        <div>
          <ModalTitle asChild>
            <h3 className="text-xl font-bold text-white">{title}</h3>
          </ModalTitle>
          <ModalDescription asChild>
            <p className="mt-2 text-sm text-muted">{subtitle}</p>
          </ModalDescription>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose}>
          Close
        </Button>
      </div>
      <div className="px-6 py-6">{children}</div>
    </ModalFrame>
  );
}
