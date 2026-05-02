import Button from "./button";

interface ProgressOverlayDialogProps {
  open: boolean;
  title: string;
  detail: string;
  progress: number;
  onClose: () => void;
  closeDisabled?: boolean;
  closeLabel?: string;
}

export default function ProgressOverlayDialog({
  open,
  title,
  detail,
  progress,
  onClose,
  closeDisabled = false,
  closeLabel = "Close",
}: ProgressOverlayDialogProps) {
  if (!open) {
    return null;
  }

  const normalizedProgress = Math.max(0, Math.min(100, progress));

  return (
    <div className="fixed inset-0 z-[80] flex items-center justify-center bg-black/85 p-6">
      <div
        role="dialog"
        aria-modal="true"
        className="w-full max-w-xl border-2 border-white bg-black p-6 shadow-[8px_8px_0_rgba(255,255,255,0.25)]"
      >
        <h3 className="text-base font-semibold text-white">{title}</h3>
        <p className="mt-3 font-mono text-xs text-muted">{detail}</p>
        <div className="mt-4 h-3 w-full overflow-hidden border border-edge-strong bg-surface-hover">
          <div
            className="h-full bg-success transition-[width] duration-300"
            style={{ width: `${normalizedProgress}%` }}
          />
        </div>
        <p className="mt-2 text-right font-mono text-xs text-muted">
          {Math.round(normalizedProgress)}%
        </p>
        <div className="mt-4 flex justify-end">
          <Button
            type="button"
            variant="secondary"
            size="sm"
            onClick={onClose}
            disabled={closeDisabled}
          >
            {closeLabel}
          </Button>
        </div>
      </div>
    </div>
  );
}
