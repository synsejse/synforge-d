import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import AppShell from "../components/common/app-shell";
import { sessionQueries } from "../lib/queries/session";
import { ApiClientError } from "../lib/api";

export const Route = createFileRoute("/_authed")({
  beforeLoad: async ({ context, location }) => {
    const status = await context.queryClient.ensureQueryData(
      sessionQueries.setupStatus(),
    );
    if (!status.initialized) {
      throw redirect({ to: "/setup" });
    }
    try {
      await context.queryClient.ensureQueryData(sessionQueries.current());
    } catch (error) {
      if (error instanceof ApiClientError && error.status === 401) {
        throw redirect({
          to: "/login",
          search: { next: location.href },
        });
      }
      throw redirect({
        to: "/login",
        search: { message: "Failed to load session." },
      });
    }
  },
  component: AuthedLayout,
});

function AuthedLayout() {
  return (
    <AppShell>
      <Outlet />
    </AppShell>
  );
}
