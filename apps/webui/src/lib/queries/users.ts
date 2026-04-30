import { queryOptions } from "@tanstack/react-query";
import api from "../api";

export const usersQueries = {
  list: () =>
    queryOptions({
      queryKey: ["users", "list"] as const,
      queryFn: () => api.listUsers(),
    }),
};
