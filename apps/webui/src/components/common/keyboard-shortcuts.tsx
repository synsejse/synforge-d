import { useEffect, useState, type ReactNode } from "react";
import { useRouter } from "@tanstack/react-router";

interface ShortcutGroup {
  label: string;
  rows: { keys: ReactNode; description: string }[];
}

const NAV_KEYS: Record<string, { to: string; label: string }> = {
  d: { to: "/", label: "Dashboard" },
  j: { to: "/jobs", label: "Jobs" },
  p: { to: "/packages", label: "Packages" },
  r: { to: "/repository", label: "Repository" },
  s: { to: "/settings", label: "Settings" },
  k: { to: "/signing", label: "Signing" },
  u: { to: "/users", label: "Users" },
  t: { to: "/statistics", label: "Statistics" },
};

export function KeyboardShortcutsProvider({ children }: { children: ReactNode }) {
  const router = useRouter();
  const [overlayOpen, setOverlayOpen] = useState(false);

  useEffect(() => {
    let pendingPrefix: "g" | null = null;
    let prefixTimeout: number | null = null;

    function clearPrefix() {
      pendingPrefix = null;
      if (prefixTimeout !== null) {
        window.clearTimeout(prefixTimeout);
        prefixTimeout = null;
      }
    }

    function isTypingTarget(target: EventTarget | null): boolean {
      if (!(target instanceof HTMLElement)) return false;
      const tag = target.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
      if (target.isContentEditable) return true;
      return false;
    }

    function focusFirstSearch() {
      const main = document.getElementById("main-content");
      if (!main) return;
      const candidate = main.querySelector<HTMLInputElement>(
        'input[type="search"], input[type="text"]',
      );
      candidate?.focus();
      candidate?.select?.();
    }

    function handleKey(event: KeyboardEvent) {
      if (event.metaKey || event.ctrlKey || event.altKey) return;
      if (isTypingTarget(event.target)) {
        if (event.key === "Escape" && pendingPrefix) clearPrefix();
        return;
      }

      if (event.key === "?" || (event.shiftKey && event.key === "/")) {
        event.preventDefault();
        setOverlayOpen((open) => !open);
        clearPrefix();
        return;
      }

      if (event.key === "Escape" && overlayOpen) {
        event.preventDefault();
        setOverlayOpen(false);
        return;
      }

      if (event.key === "/" && !pendingPrefix) {
        event.preventDefault();
        focusFirstSearch();
        return;
      }

      if (pendingPrefix === "g") {
        const target = NAV_KEYS[event.key.toLowerCase()];
        clearPrefix();
        if (target) {
          event.preventDefault();
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          router.navigate({ to: target.to as any });
        }
        return;
      }

      if (event.key === "g") {
        pendingPrefix = "g";
        prefixTimeout = window.setTimeout(clearPrefix, 1500);
        return;
      }
    }

    window.addEventListener("keydown", handleKey);
    return () => {
      window.removeEventListener("keydown", handleKey);
      clearPrefix();
    };
  }, [overlayOpen, router]);

  return (
    <>
      {children}
      {overlayOpen ? (
        <ShortcutsOverlay onClose={() => setOverlayOpen(false)} />
      ) : null}
    </>
  );
}

const SHORTCUT_GROUPS: ShortcutGroup[] = [
  {
    label: "Navigation",
    rows: [
      { keys: <Chord parts={["g", "d"]} />, description: "Dashboard" },
      { keys: <Chord parts={["g", "j"]} />, description: "Jobs" },
      { keys: <Chord parts={["g", "p"]} />, description: "Packages" },
      { keys: <Chord parts={["g", "r"]} />, description: "Repository" },
      { keys: <Chord parts={["g", "t"]} />, description: "Statistics" },
      { keys: <Chord parts={["g", "u"]} />, description: "Users" },
      { keys: <Chord parts={["g", "k"]} />, description: "Signing" },
      { keys: <Chord parts={["g", "s"]} />, description: "Settings" },
    ],
  },
  {
    label: "Page",
    rows: [
      {
        keys: <Chord parts={["/"]} />,
        description: "Focus search / filter",
      },
    ],
  },
  {
    label: "Help",
    rows: [
      { keys: <Chord parts={["?"]} />, description: "Toggle this panel" },
      { keys: <Chord parts={["Esc"]} />, description: "Close panel" },
    ],
  },
];

function ShortcutsOverlay({ onClose }: { onClose: () => void }) {
  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Keyboard shortcuts"
      className="fixed inset-0 z-[90] flex items-center justify-center bg-black/85 p-4"
      onClick={onClose}
    >
      <div
        className="relative w-full max-w-2xl border-4 border-accent-lime bg-black p-6 shadow-[8px_8px_0_rgba(191,255,0,0.25)]"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="mb-5 flex items-baseline justify-between gap-4 border-b-2 border-edge pb-3">
          <h2 className="text-xl font-bold text-white">Keyboard shortcuts</h2>
          <span className="font-mono text-[10px] uppercase tracking-[0.22em] text-soft">
            press <Chord parts={["?"]} /> to close
          </span>
        </div>
        <div className="grid gap-6 sm:grid-cols-3">
          {SHORTCUT_GROUPS.map((group) => (
            <div key={group.label}>
              <h3 className="font-mono text-[10px] font-bold uppercase tracking-[0.28em] text-accent-lime">
                {group.label}
              </h3>
              <dl className="mt-3 space-y-2">
                {group.rows.map((row, idx) => (
                  <div key={idx} className="flex items-center justify-between gap-3">
                    <dt>{row.keys}</dt>
                    <dd className="text-xs text-soft">{row.description}</dd>
                  </div>
                ))}
              </dl>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function Chord({ parts }: { parts: string[] }) {
  return (
    <span className="inline-flex items-center gap-1 font-mono text-[11px]">
      {parts.map((part, idx) => (
        <span key={idx} className="contents">
          <kbd className="border border-edge-strong bg-surface-alt px-1.5 py-0.5 font-mono text-[11px] uppercase tracking-wide text-strong shadow-[1px_1px_0_rgba(255,255,255,0.15)]">
            {part}
          </kbd>
          {idx < parts.length - 1 ? (
            <span className="text-soft">then</span>
          ) : null}
        </span>
      ))}
    </span>
  );
}
