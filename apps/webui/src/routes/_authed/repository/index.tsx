import { createFileRoute } from "@tanstack/react-router";
import RepositoryBrowserPage from "../../../features/repository/repository-browser-page";

export type RepositoryKindFilter = "all" | "rpm" | "srpm" | "log";

export interface RepositoryBrowseSearch {
  offset?: number;
  packageFilter?: string;
  targetFilter?: string;
  kindFilter?: RepositoryKindFilter;
}

function readKind(value: unknown): RepositoryKindFilter {
  return value === "rpm" || value === "srpm" || value === "log" ? value : "all";
}

export const Route = createFileRoute("/_authed/repository/")({
  validateSearch: (search: Record<string, unknown>): RepositoryBrowseSearch => ({
    offset:
      typeof search.offset === "number" ? search.offset : Number(search.offset ?? 0) || 0,
    packageFilter:
      typeof search.packageFilter === "string"
        ? search.packageFilter
        : typeof search.package === "string"
          ? search.package
          : "",
    targetFilter:
      typeof search.targetFilter === "string"
        ? search.targetFilter
        : typeof search.target === "string"
          ? search.target
          : "",
    kindFilter: readKind(search.kindFilter ?? search.kind),
  }),
  component: RepositoryBrowserPage,
});
