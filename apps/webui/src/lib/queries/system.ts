import { queryOptions } from "@tanstack/react-query";
import api from "../api";

export const systemQueries = {
  hardware: () =>
    queryOptions({
      queryKey: ["system", "hardware"] as const,
      queryFn: () => api.getServerHardware(),
      staleTime: 60_000,
    }),
};
