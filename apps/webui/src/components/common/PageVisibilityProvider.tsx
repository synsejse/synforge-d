import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";

const PageVisibilityContext = createContext<boolean | null>(null);

export function usePageVisible(): boolean {
  const value = useContext(PageVisibilityContext);
  if (value === null) {
    throw new Error(
      "usePageVisible must be used inside <PageVisibilityProvider>",
    );
  }
  return value;
}

export default function PageVisibilityProvider({
  children,
}: {
  children: ReactNode;
}) {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    if (typeof document === "undefined") return;
    const update = () => setVisible(document.visibilityState === "visible");
    update();
    document.addEventListener("visibilitychange", update);
    return () => document.removeEventListener("visibilitychange", update);
  }, []);

  return (
    <PageVisibilityContext.Provider value={visible}>
      {children}
    </PageVisibilityContext.Provider>
  );
}
