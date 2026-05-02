import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";

export type Theme = "dark" | "print";

interface ThemeContextValue {
  theme: Theme;
  toggle: () => void;
  set: (next: Theme) => void;
}

const STORAGE_KEY = "synforge.theme";

const ThemeContext = createContext<ThemeContextValue | null>(null);

function readStoredTheme(): Theme {
  if (typeof window === "undefined") return "dark";
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored === "print" || stored === "dark") return stored;
  } catch {
    /* localStorage disabled */
  }
  return "dark";
}

function applyTheme(theme: Theme) {
  if (typeof document === "undefined") return;
  if (theme === "dark") {
    document.documentElement.removeAttribute("data-theme");
  } else {
    document.documentElement.setAttribute("data-theme", theme);
  }
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<Theme>(readStoredTheme);

  useEffect(() => {
    applyTheme(theme);
    try {
      window.localStorage.setItem(STORAGE_KEY, theme);
    } catch {
      /* ignore */
    }
  }, [theme]);

  const toggle = useCallback(
    () => setTheme((prev) => (prev === "dark" ? "print" : "dark")),
    [],
  );

  const set = useCallback((next: Theme) => setTheme(next), []);

  return (
    <ThemeContext.Provider value={{ theme, toggle, set }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    // Fall back to a no-op context when used outside the provider.
    return {
      theme: "dark",
      toggle: () => {},
      set: () => {},
    };
  }
  return ctx;
}
