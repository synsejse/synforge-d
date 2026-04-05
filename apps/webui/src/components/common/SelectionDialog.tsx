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
      <div className="mx-auto w-full max-w-3xl border border-zinc-800 bg-black">
        <div className="flex items-start justify-between gap-4 border-b border-zinc-800 px-6 py-5">
          <div>
            <h3 className="text-xl font-semibold text-white">{title}</h3>
            <p className="mt-2 text-sm text-zinc-400">{subtitle}</p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="border border-zinc-800 bg-black px-3 py-2 text-sm text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-950"
          >
            Close
          </button>
        </div>
        <div className="px-6 py-6">{children}</div>
      </div>
    </div>
  );
}
