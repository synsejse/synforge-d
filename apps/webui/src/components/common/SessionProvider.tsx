import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";
import type { SessionResponse } from "../../lib/types";
import { sessionQueries } from "../../lib/queries/session";

interface SessionContextValue {
  session: SessionResponse | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

export function useSession(): SessionContextValue {
  const queryClient = useQueryClient();
  const query = useQuery(sessionQueries.current());

  const refresh = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: ["session"] });
  }, [queryClient]);

  return {
    session: query.data ?? null,
    loading: query.isPending,
    error: query.error instanceof Error ? query.error.message : null,
    refresh,
  };
}
