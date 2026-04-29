import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import api from "../../lib/api";
import type { ServerHardwareResponse } from "../../lib/types";

const ServerHardwareContext = createContext<
  ServerHardwareResponse | null | undefined
>(undefined);

export function useServerHardware(): ServerHardwareResponse | null {
  const value = useContext(ServerHardwareContext);
  if (value === undefined) {
    throw new Error(
      "useServerHardware must be used inside <ServerHardwareProvider>",
    );
  }
  return value;
}

export default function ServerHardwareProvider({
  children,
}: {
  children: ReactNode;
}) {
  const [hardware, setHardware] = useState<ServerHardwareResponse | null>(null);

  useEffect(() => {
    let cancelled = false;
    api
      .getServerHardware()
      .then((response) => {
        if (!cancelled) setHardware(response);
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <ServerHardwareContext.Provider value={hardware}>
      {children}
    </ServerHardwareContext.Provider>
  );
}
