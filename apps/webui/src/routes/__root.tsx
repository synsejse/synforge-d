import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import type { QueryClient } from "@tanstack/react-query";
import ErrorBoundary from "../components/common/error-boundary";
import DialogsProvider from "../components/common/dialogs-provider";
import ToastProvider from "../components/common/toast-provider";
import PageVisibilityProvider from "../components/common/page-visibility-provider";

export interface RouterContext {
  queryClient: QueryClient;
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootLayout,
});

function RootLayout() {
  return (
    <ErrorBoundary>
      <PageVisibilityProvider>
        <ToastProvider>
          <DialogsProvider>
            <a
              href="#main-content"
              className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-[100] focus:border focus:border-zinc-200 focus:bg-zinc-100 focus:px-4 focus:py-2 focus:text-sm focus:font-medium focus:text-black"
            >
              Skip to main content
            </a>
            <div className="fixed inset-0 -z-10 overflow-hidden">
              <div className="app-chrome absolute inset-0" />
              <div className="app-grid absolute inset-0 opacity-60" />
            </div>
            <Outlet />
          </DialogsProvider>
        </ToastProvider>
      </PageVisibilityProvider>
    </ErrorBoundary>
  );
}
