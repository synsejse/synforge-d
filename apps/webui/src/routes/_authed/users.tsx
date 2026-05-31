import { createFileRoute } from "@tanstack/react-router";
import UsersPage from "../../features/users/users-page";

export interface UsersListSearch {
  offset?: number;
}

export const Route = createFileRoute("/_authed/users")({
  validateSearch: (search: Record<string, unknown>): UsersListSearch => ({
    offset:
      typeof search.offset === "number"
        ? search.offset
        : Number(search.offset ?? 0) || 0,
  }),
  component: UsersPage,
});
