import { z } from "zod";

export type CreateFieldErrors = Partial<
  Record<"handle" | "display_name" | "password", string>
>;
export type EditFieldErrors = Partial<Record<"handle" | "display_name", string>>;

const handleSchema = z
  .string()
  .trim()
  .min(1, "Handle is required")
  .max(64, "Handle must be 64 characters or fewer")
  .regex(
    /^[a-z0-9][a-z0-9_-]*$/,
    "Use lowercase letters, digits, underscore, or hyphen",
  );
const displayNameSchema = z
  .string()
  .trim()
  .min(1, "Display name is required")
  .max(120, "Display name is too long");
export const passwordSchema = z
  .string()
  .min(8, "Password must be at least 8 characters");

export const createUserSchema = z.object({
  handle: handleSchema,
  display_name: displayNameSchema,
  password: passwordSchema,
});
export const editUserSchema = z.object({
  handle: handleSchema,
  display_name: displayNameSchema,
});

export function flatErrors<T extends string>(result: {
  error: { issues: Array<{ path: PropertyKey[]; message: string }> };
}): Partial<Record<T, string>> {
  const out: Partial<Record<T, string>> = {};
  for (const issue of result.error.issues) {
    const key = issue.path[0];
    if (typeof key === "string" && !(key in out)) {
      (out as Record<string, string>)[key] = issue.message;
    }
  }
  return out;
}
