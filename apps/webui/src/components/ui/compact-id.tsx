import { useEffect, useRef, useState } from "react";
import { faCheck, faCopy } from "@fortawesome/free-solid-svg-icons";
import { formatCompactId } from "../../lib/identifiers";
import Button from "./button";
import FaIcon from "./fa-icon";

interface Props {
  value: string;
  copyable?: boolean;
  className?: string;
}

export default function CompactId({
  value,
  copyable = true,
  className = "",
}: Props) {
  const [copied, setCopied] = useState(false);
  const resetTimer = useRef<number | null>(null);

  useEffect(
    () => () => {
      if (resetTimer.current != null) window.clearTimeout(resetTimer.current);
    },
    [],
  );

  async function copy() {
    if (!navigator.clipboard) return;
    try {
      await navigator.clipboard.writeText(value);
    } catch {
      return;
    }
    setCopied(true);
    if (resetTimer.current != null) window.clearTimeout(resetTimer.current);
    resetTimer.current = window.setTimeout(() => setCopied(false), 1500);
  }

  return (
    <span className={`inline-flex min-w-0 items-center gap-1.5 ${className}`}>
      <code className="truncate font-mono text-inherit" title={value}>
        {formatCompactId(value)}
      </code>
      {copyable ? (
        <Button
          variant="subtle"
          size="icon-sm"
          onClick={() => void copy()}
          aria-label={copied ? "Identifier copied" : `Copy identifier ${value}`}
          title={copied ? "Copied" : "Copy full identifier"}
          className="shrink-0"
        >
          <FaIcon icon={copied ? faCheck : faCopy} />
        </Button>
      ) : null}
    </span>
  );
}
