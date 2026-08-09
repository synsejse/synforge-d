import { useEffect, useState, type ReactNode } from "react";
import { PageVisibilityContext } from "./page-visibility-context";

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
