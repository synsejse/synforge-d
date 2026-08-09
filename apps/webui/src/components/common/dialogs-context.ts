import { createContext, useContext } from "react";

export interface ConfirmOptions {
  title: string;
  message?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  destructive?: boolean;
}

export interface DialogsContextValue {
  confirm: (options: ConfirmOptions) => Promise<boolean>;
}

export const DialogsContext = createContext<DialogsContextValue | null>(null);

export function useDialogs(): DialogsContextValue {
  const value = useContext(DialogsContext);
  if (!value) {
    throw new Error("useDialogs must be used inside <DialogsProvider>");
  }
  return value;
}
