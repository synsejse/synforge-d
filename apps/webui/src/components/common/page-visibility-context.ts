import { createContext, useContext } from "react";

export const PageVisibilityContext = createContext<boolean | null>(null);

export function usePageVisible(): boolean {
  const value = useContext(PageVisibilityContext);
  if (value === null) {
    throw new Error(
      "usePageVisible must be used inside <PageVisibilityProvider>",
    );
  }
  return value;
}
