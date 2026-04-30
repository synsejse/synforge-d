import { createFileRoute } from "@tanstack/react-router";
import PackageListPage from "../../../features/packages/package-list-page";

export interface PackagesListSearch {
  offset?: number;
  search?: string;
  enabled?: "all" | "true" | "false";
}

export const Route = createFileRoute("/_authed/packages/")({
  validateSearch: (search: Record<string, unknown>): PackagesListSearch => {
    const enabled = search.enabled === "true" || search.enabled === "false"
      ? (search.enabled as "true" | "false")
      : "all";
    return {
      offset:
        typeof search.offset === "number"
          ? search.offset
          : Number(search.offset ?? 0) || 0,
      search: typeof search.search === "string" ? search.search : "",
      enabled,
    };
  },
  component: PackageListPage,
});
