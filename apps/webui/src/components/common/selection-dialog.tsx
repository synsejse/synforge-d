import type { ReactNode } from "react";
import Button from "../ui/button";

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
    <div className="fixed inset-0 z-[60] overflow-y-auto bg-black/80 px-4 py-8">
      <div className="mx-auto w-full max-w-3xl border-4 border-white bg-black shadow-[6px_6px_0_rgba(255,255,255,0.25)]">
        <div className="flex items-start justify-between gap-4 border-b-2 border-edge px-6 py-5">
          <div>
            <h3 className="font-mono text-xl font-bold uppercase text-white">{title}</h3>
            <p className="mt-2 text-sm text-muted">{subtitle}</p>
          </div>
          <Button variant="ghost" size="sm" onClick={onClose}>
            Close
          </Button>
        </div>
        <div className="px-6 py-6">{children}</div>
      </div>
    </div>
  );
}
