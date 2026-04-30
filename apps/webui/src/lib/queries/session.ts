import { queryOptions } from "@tanstack/react-query";
import api from "../api";

export const sessionQueries = {
  current: () =>
    queryOptions({
      queryKey: ["session"] as const,
      queryFn: () => api.getSession(),
      staleTime: 30_000,
      retry: false,
    }),
  setupStatus: () =>
    queryOptions({
      queryKey: ["session", "setup-status"] as const,
      queryFn: () => api.getSetupStatus(),
      staleTime: 60_000,
    }),
};
