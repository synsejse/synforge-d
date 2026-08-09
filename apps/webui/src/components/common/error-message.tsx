import { faRotate } from "@fortawesome/free-solid-svg-icons";
import Button from "../ui/button";
import FaIcon from "../ui/fa-icon";

interface ErrorMessageProps {
  message: string;
  onRetry?: () => void;
  retrying?: boolean;
}

/**
 * Brutalist error block. Brief glitch on first paint — the [ERR] tag
 * displaces and recovers once. Plays once via the synforge-glitch-once
 * CSS class; the keyframe is a no-op when prefers-reduced-motion is set.
 */
export default function ErrorMessage({
  message,
  onRetry,
  retrying = false,
}: ErrorMessageProps) {
  return (
    <div role="alert" className="border border-error bg-black p-4">
      <div className="flex items-center gap-2">
        <span className="synforge-glitch-once font-mono text-xs font-bold uppercase tracking-[0.18em] text-error">
          [ERR]
        </span>
        <span
          aria-hidden="true"
          className="h-px flex-1 bg-error/40"
        />
      </div>
      <p className="mt-2 text-sm text-strong">{message}</p>
      {onRetry ? (
        <Button
          variant="ghost"
          size="sm"
          onClick={onRetry}
          loading={retrying}
          className="mt-4 border-error/60 text-error hover:border-error hover:text-error"
        >
          {retrying ? null : <FaIcon icon={faRotate} />}
          {retrying ? "Retrying…" : "Retry"}
        </Button>
      ) : null}
    </div>
  );
}
