import type { ReactNode } from "react";
import ErrorBoundary from "./ErrorBoundary";
import DialogsProvider from "./DialogsProvider";

export default function PageRoot({ children }: { children: ReactNode }) {
  return (
    <ErrorBoundary>
      <DialogsProvider>{children}</DialogsProvider>
    </ErrorBoundary>
  );
}
