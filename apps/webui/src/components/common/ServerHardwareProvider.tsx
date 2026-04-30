import { useQuery } from "@tanstack/react-query";
import type { ServerHardwareResponse } from "../../lib/types";
import { systemQueries } from "../../lib/queries";

export function useServerHardware(): ServerHardwareResponse | null {
  const query = useQuery(systemQueries.hardware());
  return query.data ?? null;
}
