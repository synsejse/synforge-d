import { createFileRoute } from "@tanstack/react-router";
import PackageDetail from "../../../features/packages/components/package-detail";

interface PackageViewSearch {
  name?: string;
}

export const Route = createFileRoute("/_authed/packages/view")({
  validateSearch: (search: Record<string, unknown>): PackageViewSearch => ({
    name: typeof search.name === "string" ? search.name : undefined,
  }),
  component: PackageView,
});

function PackageView() {
  const { name } = Route.useSearch();
  if (!name) {
    return (
      <div className="border-2 border-[var(--theme-error-red)] bg-black px-4 py-3 font-mono text-sm text-[var(--theme-error-red)]">
        Invalid package name
      </div>
    );
  }
  return <PackageDetail packageName={name} />;
}
