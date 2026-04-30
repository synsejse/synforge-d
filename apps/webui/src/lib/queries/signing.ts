import { queryOptions } from "@tanstack/react-query";
import api from "../api";

export const signingQueries = {
  status: () =>
    queryOptions({
      queryKey: ["signing", "status"] as const,
      queryFn: () => api.getRepoSigningStatus(),
    }),
  reconcileProgress: () =>
    queryOptions({
      queryKey: ["signing", "reconcile-progress"] as const,
      queryFn: () => api.getRepoSigningReconcileProgress(),
    }),
};
