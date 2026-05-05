import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatMockChroots(
  chroots: string[] | undefined,
  emptyLabel = "None",
): string {
  if (!chroots || chroots.length === 0) {
    return emptyLabel;
  }
  return chroots.join(", ");
}

/** Architectures Mock's ccache plugin understands. */
export const CCACHE_SUPPORTED_ARCHES = [
  "aarch64",
  "ppc64le",
  "s390x",
  "src",
  "x86_64",
] as const;

export function incompatibleCcacheChroots(chroots: string[]): string[] {
  return chroots.filter((chroot) => {
    const arch = chroot.split("-").pop();
    return !arch || !CCACHE_SUPPORTED_ARCHES.includes(
      arch as (typeof CCACHE_SUPPORTED_ARCHES)[number],
    );
  });
}
