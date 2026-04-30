import { queryOptions } from "@tanstack/react-query";
import api from "../api";

export const configQueries = {
  effective: () =>
    queryOptions({
      queryKey: ["config", "effective"] as const,
      queryFn: () => api.getConfig(),
    }),
  schema: () =>
    queryOptions({
      queryKey: ["config", "schema"] as const,
      queryFn: () => api.getConfigSchema(),
    }),
};
