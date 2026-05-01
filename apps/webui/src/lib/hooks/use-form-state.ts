import { useCallback, useState } from "react";
import { z, type ZodType } from "zod";

export interface FormState<T> {
  values: T;
  errors: Partial<Record<keyof T, string>>;
  setField: <K extends keyof T>(key: K, value: T[K]) => void;
  setValues: (next: T | ((prev: T) => T)) => void;
  validate: () => { ok: true; data: T } | { ok: false; errors: Partial<Record<keyof T, string>> };
  reset: (next?: T) => void;
  /** True if validate() has been called at least once and produced errors. */
  attempted: boolean;
}

/**
 * Tiny form-state hook. Holds typed values, exposes setField for
 * inputs, and runs a zod schema on demand. On validation failure the
 * `errors` map carries one message per field path.
 *
 * Inputs read errors via `errors[field]` and render inline; validate()
 * gates the submit. Errors clear when the user edits the offending
 * field.
 */
export function useFormState<T extends Record<string, unknown>>(
  initial: T,
  schema: ZodType<T>,
): FormState<T> {
  const [values, setValuesState] = useState<T>(initial);
  const [errors, setErrors] = useState<Partial<Record<keyof T, string>>>({});
  const [attempted, setAttempted] = useState(false);

  const setField = useCallback(<K extends keyof T>(key: K, value: T[K]) => {
    setValuesState((prev) => ({ ...prev, [key]: value }));
    setErrors((prev) => {
      if (!prev[key]) return prev;
      const next = { ...prev };
      delete next[key];
      return next;
    });
  }, []);

  const setValues = useCallback((next: T | ((prev: T) => T)) => {
    setValuesState((prev) =>
      typeof next === "function" ? (next as (p: T) => T)(prev) : next,
    );
  }, []);

  const validate = useCallback(() => {
    const result = schema.safeParse(values);
    setAttempted(true);
    if (result.success) {
      setErrors({});
      return { ok: true as const, data: result.data };
    }
    const flat: Partial<Record<keyof T, string>> = {};
    for (const issue of result.error.issues) {
      const path = issue.path[0];
      if (typeof path === "string" && !(path in flat)) {
        flat[path as keyof T] = issue.message;
      }
    }
    setErrors(flat);
    return { ok: false as const, errors: flat };
  }, [schema, values]);

  const reset = useCallback((next?: T) => {
    setValuesState(next ?? initial);
    setErrors({});
    setAttempted(false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return { values, errors, setField, setValues, validate, reset, attempted };
}

export { z };
