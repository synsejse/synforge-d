import type { ReactNode } from "react";
import Button from "./button";
import ModalFrame, { ModalTitle } from "./modal-frame";

interface ProgressOverlayDialogProps {
  open: boolean;
  title: string;
  /** Optional accent — controls the title colour and progress-bar fill.
   *  Defaults to the success/lime fill used while running. */
  tone?: "running" | "success" | "error";
  /** Optional one-line summary directly under the title (e.g.
   *  "12 / 30 packages"). For richer content, use `children`. */
  summary?: ReactNode;
  progress: number;
  /** Anything to render below the progress bar and above the close
   *  button — stat grid, status message, etc. */
  children?: ReactNode;
  onClose: () => void;
  closeDisabled?: boolean;
  closeLabel?: string;
}

const TITLE_TONE: Record<NonNullable<ProgressOverlayDialogProps["tone"]>, string> = {
  running: "text-accent-lime",
  success: "text-success",
  error: "text-error",
};

const BAR_TONE: Record<NonNullable<ProgressOverlayDialogProps["tone"]>, string> = {
  running: "bg-accent-lime",
  success: "bg-success",
  error: "bg-error",
};

export default function ProgressOverlayDialog({
  open,
  title,
  tone = "running",
  summary,
  progress,
  children,
  onClose,
  closeDisabled = false,
  closeLabel = "Close",
}: ProgressOverlayDialogProps) {
  const normalizedProgress = Math.max(0, Math.min(100, progress));

  return (
    <ModalFrame
      open={open}
      // Non-dismissable by design: Escape / outside-click can't close it, but
      // focus is still trapped inside while a refresh-all run is in flight.
      onOpenChange={() => undefined}
      dismissable={false}
      zIndex={80}
      overlayClassName="flex items-center justify-center bg-black/85 p-6"
      className="max-w-xl border border-edge bg-black shadow-card-lg"
    >
      {/* Header band — coloured by tone, mirrors the page-header
          accent vocabulary used elsewhere in the app. */}
      <div className="border-b border-edge bg-surface-alt px-5 py-3">
        <ModalTitle asChild>
          <h3
            className={`font-mono text-xs font-bold uppercase tracking-[0.22em] ${TITLE_TONE[tone]}`}
          >
            {title}
          </h3>
        </ModalTitle>
      </div>

      <div className="px-5 py-5">
        {summary !== undefined ? (
          <div className="mb-3 font-mono text-2xl font-black uppercase tracking-tight text-white">
            {summary}
          </div>
        ) : null}

        <div className="h-3 w-full border border-edge bg-black">
          <div
            className={`h-full transition-[width] duration-300 ${BAR_TONE[tone]}`}
            style={{ width: `${normalizedProgress}%` }}
          />
        </div>

        {children !== undefined ? (
          <div className="mt-5">{children}</div>
        ) : null}
      </div>

      <div className="flex justify-end border-t border-edge bg-surface-alt px-5 py-3">
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
    </ModalFrame>
  );
}
