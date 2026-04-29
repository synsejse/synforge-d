import {
  createContext,
  useCallback,
  useContext,
  useState,
  type ReactNode,
} from "react";
import Dialog from "../ui/Dialog";
import Button from "../ui/Button";

interface ConfirmOptions {
  title: string;
  message?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  destructive?: boolean;
}

interface NotifyOptions {
  title: string;
  message: string;
  variant?: "info" | "error";
}

interface DialogsContextValue {
  confirm: (options: ConfirmOptions) => Promise<boolean>;
  notify: (options: NotifyOptions) => Promise<void>;
}

interface ConfirmState {
  options: ConfirmOptions;
  resolve: (value: boolean) => void;
}

interface NotifyState {
  options: NotifyOptions;
  resolve: () => void;
}

const DialogsContext = createContext<DialogsContextValue | null>(null);

export function useDialogs(): DialogsContextValue {
  const value = useContext(DialogsContext);
  if (!value) {
    throw new Error("useDialogs must be used inside <DialogsProvider>");
  }
  return value;
}

export default function DialogsProvider({ children }: { children: ReactNode }) {
  const [confirmState, setConfirmState] = useState<ConfirmState | null>(null);
  const [notifyState, setNotifyState] = useState<NotifyState | null>(null);

  const confirm = useCallback(
    (options: ConfirmOptions) =>
      new Promise<boolean>((resolve) => {
        setConfirmState({ options, resolve });
      }),
    [],
  );

  const notify = useCallback(
    (options: NotifyOptions) =>
      new Promise<void>((resolve) => {
        setNotifyState({ options, resolve });
      }),
    [],
  );

  const closeConfirm = (value: boolean) => {
    if (!confirmState) {
      return;
    }
    confirmState.resolve(value);
    setConfirmState(null);
  };

  const closeNotify = () => {
    if (!notifyState) {
      return;
    }
    notifyState.resolve();
    setNotifyState(null);
  };

  return (
    <DialogsContext.Provider value={{ confirm, notify }}>
      {children}
      {confirmState ? (
        <Dialog
          open
          onOpenChange={(open) => {
            if (!open) closeConfirm(false);
          }}
          title={confirmState.options.title}
          description={confirmState.options.message}
          showClose={false}
        >
          <div className="flex justify-end gap-3">
            <Button
              type="button"
              variant="ghost"
              size="md"
              onClick={() => closeConfirm(false)}
            >
              {confirmState.options.cancelLabel ?? "Cancel"}
            </Button>
            <Button
              type="button"
              variant={confirmState.options.destructive ? "danger" : "primary"}
              size="md"
              onClick={() => closeConfirm(true)}
              autoFocus
            >
              {confirmState.options.confirmLabel ?? "Confirm"}
            </Button>
          </div>
        </Dialog>
      ) : null}
      {notifyState ? (
        <Dialog
          open
          onOpenChange={(open) => {
            if (!open) closeNotify();
          }}
          title={notifyState.options.title}
          showClose={false}
        >
          <p
            className={
              notifyState.options.variant === "error"
                ? "font-mono text-sm text-[var(--theme-error-red)]"
                : "font-mono text-sm text-zinc-200"
            }
          >
            {notifyState.options.message}
          </p>
          <div className="mt-6 flex justify-end">
            <Button
              type="button"
              variant="primary"
              size="md"
              onClick={closeNotify}
              autoFocus
            >
              OK
            </Button>
          </div>
        </Dialog>
      ) : null}
    </DialogsContext.Provider>
  );
}
