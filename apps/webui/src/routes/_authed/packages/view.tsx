import { createFileRoute } from "@tanstack/react-router";
import PackageDetailPage from "../../../features/packages/PackageDetailPage";

interface PackageViewSearch {
  name?: string;
}

export const Route = createFileRoute("/_authed/packages/view")({
  validateSearch: (search: Record<string, unknown>): PackageViewSearch => ({
    name: typeof search.name === "string" ? search.name : undefined,
  }),
  component: PackageDetailPage,
});
