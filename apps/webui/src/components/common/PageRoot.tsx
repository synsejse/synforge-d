import type { ReactNode } from "react";
import ErrorBoundary from "./ErrorBoundary";
import DialogsProvider from "./DialogsProvider";
import QueryProvider from "./QueryProvider";

export default function PageRoot({ children }: { children: ReactNode }) {
  return (
    <ErrorBoundary>
      <QueryProvider>
        <DialogsProvider>{children}</DialogsProvider>
      </QueryProvider>
    </ErrorBoundary>
  );
}
