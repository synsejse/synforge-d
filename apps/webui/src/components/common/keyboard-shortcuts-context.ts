import { createContext, useContext } from "react";

export interface KeyboardShortcutsValue {
  open: () => void;
}

export const KeyboardShortcutsContext =
  createContext<KeyboardShortcutsValue | null>(null);

export function useKeyboardShortcuts(): KeyboardShortcutsValue {
  const value = useContext(KeyboardShortcutsContext);
  if (!value) {
    throw new Error(
      "useKeyboardShortcuts must be used inside KeyboardShortcutsProvider",
    );
  }
  return value;
}
