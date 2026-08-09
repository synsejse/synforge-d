import { useBlocker } from "@tanstack/react-router";

const DEFAULT_MESSAGE =
  "You have unsaved changes. Leave this page and discard them?";

export default function useUnsavedChangesWarning(
  enabled: boolean,
  message = DEFAULT_MESSAGE,
) {
  useBlocker({
    disabled: !enabled,
    enableBeforeUnload: enabled,
    shouldBlockFn: () => !window.confirm(message),
  });
}
