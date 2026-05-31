import { queryOptions } from "@tanstack/react-query";
import api from "../api";

export interface UsersListParams {
  limit: number;
  offset: number;
}

export const usersQueries = {
  list: (params: UsersListParams) =>
    queryOptions({
      queryKey: ["users", "list", params] as const,
      queryFn: () => api.listUsers(params.limit, params.offset),
      placeholderData: (previous) => previous,
    }),
};
