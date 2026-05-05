import { useCallback, useMemo, useState } from "react";

export interface SelectionApi<K> {
  selected: Set<K>;
  count: number;
  isSelected: (key: K) => boolean;
  toggle: (key: K) => void;
  setOne: (key: K, value: boolean) => void;
  setMany: (keys: K[], value: boolean) => void;
  allSelected: (keys: K[]) => boolean;
  someSelected: (keys: K[]) => boolean;
  clear: () => void;
}

export function useSelection<K>(): SelectionApi<K> {
  const [selected, setSelected] = useState<Set<K>>(() => new Set());

  const isSelected = useCallback(
    (key: K) => selected.has(key),
    [selected],
  );

  const toggle = useCallback((key: K) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }, []);

  const setOne = useCallback((key: K, value: boolean) => {
    setSelected((prev) => {
      const has = prev.has(key);
      if (has === value) return prev;
      const next = new Set(prev);
      if (value) next.add(key);
      else next.delete(key);
      return next;
    });
  }, []);

  const setMany = useCallback((keys: K[], value: boolean) => {
    setSelected((prev) => {
      const next = new Set(prev);
      for (const k of keys) {
        if (value) next.add(k);
        else next.delete(k);
      }
      return next;
    });
  }, []);

  const allSelected = useCallback(
    (keys: K[]) => {
      if (keys.length === 0) return false;
      for (const k of keys) {
        if (!selected.has(k)) return false;
      }
      return true;
    },
    [selected],
  );

  const someSelected = useCallback(
    (keys: K[]) => {
      let count = 0;
      for (const k of keys) {
        if (selected.has(k)) count++;
        if (count > 0 && count < keys.length) return true;
      }
      return false;
    },
    [selected],
  );

  const clear = useCallback(() => {
    setSelected((prev) => (prev.size === 0 ? prev : new Set()));
  }, []);

  return useMemo(
    () => ({
      selected,
      count: selected.size,
      isSelected,
      toggle,
      setOne,
      setMany,
      allSelected,
      someSelected,
      clear,
    }),
    [selected, isSelected, toggle, setOne, setMany, allSelected, someSelected, clear],
  );
}
