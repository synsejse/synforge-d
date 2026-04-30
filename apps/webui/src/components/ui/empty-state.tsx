import type { ReactNode } from "react";

interface Props {
  children: ReactNode;
}

export default function EmptyState({ children }: Props) {
  return (
    <div className="border-2 border-dashed border-zinc-700 bg-black p-8 text-center font-mono text-sm text-zinc-400">
      {children}
    </div>
  );
}
