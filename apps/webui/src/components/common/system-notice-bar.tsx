import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { faTriangleExclamation, faXmark } from "@fortawesome/free-solid-svg-icons";
import { signingQueries } from "../../lib/queries";
import FaIcon from "../ui/fa-icon";

interface Notice {
  id: string;
  text: string;
}

const DISMISSED_STORAGE_KEY = "synforge.dismissedNotices";

function readDismissed(): Set<string> {
  if (typeof window === "undefined") return new Set();
  try {
    const raw = window.localStorage.getItem(DISMISSED_STORAGE_KEY);
    if (!raw) return new Set();
    const arr = JSON.parse(raw);
    if (Array.isArray(arr)) return new Set(arr.map(String));
  } catch {
    /* ignore */
  }
  return new Set();
}

function writeDismissed(set: Set<string>) {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(
      DISMISSED_STORAGE_KEY,
      JSON.stringify(Array.from(set)),
    );
  } catch {
    /* ignore */
  }
}

/**
 * Top-of-app marquee that surfaces daemon-level notices. Currently
 * derives notices from signing status; the list is user-dismissible
 * (per-id, persisted in localStorage). When the resolved list is
 * empty the bar renders nothing.
 *
 * The marquee scrolls left at ~60s per cycle; on hover the animation
 * pauses so users can read.
 */
export default function SystemNoticeBar() {
  const signingQuery = useQuery({ ...signingQueries.status(), retry: false });
  const [dismissed, setDismissed] = useState<Set<string>>(readDismissed);

  const notices: Notice[] = [];
  const signing = signingQuery.data?.status;
  if (signing?.enabled && !signing.key_present) {
    notices.push({
      id: "signing-key-missing",
      text:
        "Signing is enabled but no key is loaded — generate or import a key, or disable signing.",
    });
  }

  const visible = notices.filter((n) => !dismissed.has(n.id));
  if (visible.length === 0) return null;

  function dismiss(id: string) {
    setDismissed((prev) => {
      const next = new Set(prev);
      next.add(id);
      writeDismissed(next);
      return next;
    });
  }

  return (
    <div
      role="status"
      aria-live="polite"
      className="border-b-2 border-accent-orange bg-black"
    >
      {visible.map((notice) => (
        <div
          key={notice.id}
          className="group relative flex items-center gap-3 overflow-hidden px-3 py-1.5"
        >
          <span className="flex shrink-0 items-center gap-2 font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-accent-orange">
            <FaIcon icon={faTriangleExclamation} />
            Notice
          </span>
          <div className="relative min-w-0 flex-1 overflow-hidden">
            <div
              className="synforge-marquee whitespace-nowrap font-mono text-xs text-strong"
              style={{ animationDuration: "60s" }}
            >
              <span>{notice.text}</span>
              <span className="mx-12 text-soft" aria-hidden="true">
                ◆
              </span>
              <span>{notice.text}</span>
            </div>
          </div>
          <button
            type="button"
            onClick={() => dismiss(notice.id)}
            aria-label="Dismiss notice"
            className="shrink-0 border border-edge bg-black px-1.5 py-0.5 text-soft transition-colors hover:border-white hover:text-white"
          >
            <FaIcon icon={faXmark} className="text-[11px]" />
          </button>
        </div>
      ))}
    </div>
  );
}
