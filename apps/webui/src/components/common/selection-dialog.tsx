import type { ReactNode } from "react";

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
          <button
            type="button"
            onClick={onClose}
            className="border-2 border-edge-strong bg-black px-3 py-2 font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:border-white hover:bg-surface-alt"
          >
            Close
          </button>
        </div>
        <div className="px-6 py-6">{children}</div>
      </div>
    </div>
  );
}
