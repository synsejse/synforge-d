import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Utility for merging Tailwind classes with proper precedence
 * Combines clsx and tailwind-merge for optimal class composition
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
