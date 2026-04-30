import {
  createContext,
  useCallback,
  useContext,
  useState,
  type ReactNode,
} from "react";
import Dialog from "../ui/dialog";
import Button from "../ui/button";

interface ConfirmOptions {
  title: string;
  message?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  destructive?: boolean;
}

interface DialogsContextValue {
  confirm: (options: ConfirmOptions) => Promise<boolean>;
}

interface ConfirmState {
  options: ConfirmOptions;
  resolve: (value: boolean) => void;
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

  const confirm = useCallback(
    (options: ConfirmOptions) =>
      new Promise<boolean>((resolve) => {
        setConfirmState({ options, resolve });
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

  return (
    <DialogsContext.Provider value={{ confirm }}>
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
    </DialogsContext.Provider>
  );
}
