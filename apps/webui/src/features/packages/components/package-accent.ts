const PALETTE: ReadonlyArray<{ var: string; name: string }> = [
  { var: "var(--theme-accent-lime)", name: "lime" },
  { var: "var(--theme-accent-orange)", name: "orange" },
  { var: "var(--theme-accent-cyan)", name: "cyan" },
  { var: "var(--theme-accent-violet)", name: "violet" },
  { var: "var(--theme-terminal-green)", name: "green" },
  { var: "var(--theme-text-strong)", name: "white" },
];

function djb2(input: string): number {
  let hash = 5381;
  for (let i = 0; i < input.length; i++) {
    hash = ((hash << 5) + hash + input.charCodeAt(i)) | 0;
  }
  return hash >>> 0;
}

export function packageAccent(name: string): string {
  if (!name) return PALETTE[0].var;
  return PALETTE[djb2(name) % PALETTE.length].var;
}
