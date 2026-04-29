import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import api from "../../lib/api";
import type { SessionResponse } from "../../lib/types";

interface SessionContextValue {
  session: SessionResponse | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

const SessionContext = createContext<SessionContextValue | null>(null);

const SESSION_LOADED_EVENT = "synforge:session-loaded";
const FALLBACK_FETCH_DELAY_MS = 300;

export function useSession(): SessionContextValue {
  const value = useContext(SessionContext);
  if (!value) {
    throw new Error("useSession must be used inside <SessionProvider>");
  }
  return value;
}

function readCachedSession(): SessionResponse | null {
  if (typeof document === "undefined") return null;
  const raw = document.body.dataset.session;
  if (!raw) return null;
  try {
    return JSON.parse(raw) as SessionResponse;
  } catch {
    return null;
  }
}

export default function SessionProvider({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<SessionResponse | null>(() =>
    readCachedSession(),
  );
  const [loading, setLoading] = useState(() => readCachedSession() === null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const response = await api.getSession();
      setSession(response);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load session");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const handler = (event: Event) => {
      const detail = (event as CustomEvent<SessionResponse>).detail;
      if (detail) {
        setSession(detail);
        setError(null);
        setLoading(false);
      }
    };
    window.addEventListener(SESSION_LOADED_EVENT, handler);

    let cancelled = false;
    const fallback = window.setTimeout(() => {
      if (cancelled) return;
      if (readCachedSession() === null && session === null) {
        void refresh();
      }
    }, FALLBACK_FETCH_DELAY_MS);

    return () => {
      cancelled = true;
      window.clearTimeout(fallback);
      window.removeEventListener(SESSION_LOADED_EVENT, handler);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refresh]);

  return (
    <SessionContext.Provider value={{ session, loading, error, refresh }}>
      {children}
    </SessionContext.Provider>
  );
}
