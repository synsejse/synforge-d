import { useEffect, useState } from "react";

/**
 * Returns a debounced echo of `value` that only updates after `delay` ms
 * without further changes. Useful for piping a typing-driven filter into a
 * query without firing one request per keystroke.
 */
export function useDebounce<T>(value: T, delay = 250): T {
  const [debounced, setDebounced] = useState(value);

  useEffect(() => {
    const handle = window.setTimeout(() => setDebounced(value), delay);
    return () => window.clearTimeout(handle);
  }, [value, delay]);

  return debounced;
}
