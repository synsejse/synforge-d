import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Utility for merging Tailwind classes with proper precedence
 * Combines clsx and tailwind-merge for optimal class composition
 */
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

/**
 * Architectures Mock's ccache plugin understands. Builds for any other
 * arch (e.g. `noarch`, cross-arch via qemu) silently bypass ccache or
 * fail outright depending on the Mock config — surface a warning when
 * a package toggles ccache on but selects an incompatible target.
 */
export const CCACHE_SUPPORTED_ARCHES = [
  "aarch64",
  "ppc64le",
  "s390x",
  "src",
  "x86_64",
] as const;

/** Returns selected mock chroots whose trailing arch is not on the
 *  ccache-supported list. The mock_chroot format is
 *  `<distro>-<version>-<arch>`; we split on `-` and look at the tail. */
export function incompatibleCcacheChroots(chroots: string[]): string[] {
  return chroots.filter((chroot) => {
    const arch = chroot.split("-").pop();
    return !arch || !CCACHE_SUPPORTED_ARCHES.includes(
      arch as (typeof CCACHE_SUPPORTED_ARCHES)[number],
    );
  });
}
