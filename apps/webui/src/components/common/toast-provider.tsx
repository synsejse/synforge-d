import * as ToastPrimitive from "@radix-ui/react-toast";
import {
  createContext,
  useCallback,
  useContext,
  useState,
  type ReactNode,
} from "react";
import { cn } from "../../lib/utils";

export type ToastVariant = "info" | "success" | "error" | "warning";

interface ToastOptions {
  title: string;
  message?: string;
  variant?: ToastVariant;
  duration?: number;
}

interface ToastEntry extends ToastOptions {
  id: number;
}

interface ToastContextValue {
  toast: (options: ToastOptions) => void;
  info: (title: string, message?: string) => void;
  success: (title: string, message?: string) => void;
  error: (title: string, message?: string) => void;
  warning: (title: string, message?: string) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

export function useToast(): ToastContextValue {
  const value = useContext(ToastContext);
  if (!value) {
    throw new Error("useToast must be used inside <ToastProvider>");
  }
  return value;
}

const variantBorder: Record<ToastVariant, string> = {
  info: "border-edge-strong",
  success: "border-success",
  error: "border-error",
  warning: "border-accent-orange",
};

const variantAccent: Record<ToastVariant, string> = {
  info: "text-accent-lime",
  success: "text-success",
  error: "text-error",
  warning: "text-accent-orange",
};

const DEFAULT_DURATION = 5000;

export default function ToastProvider({ children }: { children: ReactNode }) {
  const [entries, setEntries] = useState<ToastEntry[]>([]);

  const toast = useCallback((options: ToastOptions) => {
    const id = Date.now() + Math.random();
    setEntries((prev) => [...prev, { ...options, id }]);
  }, []);

  const removeEntry = useCallback((id: number) => {
    setEntries((prev) => prev.filter((e) => e.id !== id));
  }, []);

  const info = useCallback(
    (title: string, message?: string) => toast({ title, message, variant: "info" }),
    [toast],
  );
  const success = useCallback(
    (title: string, message?: string) =>
      toast({ title, message, variant: "success" }),
    [toast],
  );
  const error = useCallback(
    (title: string, message?: string) =>
      toast({ title, message, variant: "error", duration: 8000 }),
    [toast],
  );
  const warning = useCallback(
    (title: string, message?: string) =>
      toast({ title, message, variant: "warning" }),
    [toast],
  );

  return (
    <ToastContext.Provider value={{ toast, info, success, error, warning }}>
      <ToastPrimitive.Provider swipeDirection="right">
        {children}
        {entries.map((entry) => {
          const variant = entry.variant ?? "info";
          return (
            <ToastPrimitive.Root
              key={entry.id}
              duration={entry.duration ?? DEFAULT_DURATION}
              onOpenChange={(open) => {
                if (!open) removeEntry(entry.id);
              }}
              className={cn(
                "border-2 bg-black p-4 shadow-card-md",
                "data-[state=open]:animate-in data-[state=closed]:animate-out",
                "data-[state=closed]:fade-out-80",
                "data-[state=open]:slide-in-from-right-full",
                "data-[state=closed]:slide-out-to-right-full",
                "data-[swipe=move]:translate-x-[var(--radix-toast-swipe-move-x)]",
                "data-[swipe=cancel]:translate-x-0 data-[swipe=cancel]:transition-[transform_200ms_ease-out]",
                "data-[swipe=end]:animate-out data-[swipe=end]:slide-out-to-right-full",
                variantBorder[variant],
              )}
            >
              <div className="flex items-start gap-3">
                <div className="min-w-0 flex-1">
                  <ToastPrimitive.Title
                    className={cn(
                      "font-mono text-xs font-bold uppercase tracking-[0.2em]",
                      variantAccent[variant],
                    )}
                  >
                    {entry.title}
                  </ToastPrimitive.Title>
                  {entry.message ? (
                    <ToastPrimitive.Description className="mt-1 font-mono text-sm text-strong">
                      {entry.message}
                    </ToastPrimitive.Description>
                  ) : null}
                </div>
                <ToastPrimitive.Close
                  aria-label="Close"
                  className="shrink-0 border border-edge bg-black px-2 py-1 font-mono text-xs uppercase tracking-[0.15em] text-muted transition hover:border-white hover:text-white focus:outline-none focus:ring-2 focus:ring-accent-lime"
                >
                  ×
                </ToastPrimitive.Close>
              </div>
            </ToastPrimitive.Root>
          );
        })}
        <ToastPrimitive.Viewport className="fixed bottom-4 right-4 z-[100] flex w-full max-w-md flex-col gap-3 outline-none" />
      </ToastPrimitive.Provider>
    </ToastContext.Provider>
  );
}
