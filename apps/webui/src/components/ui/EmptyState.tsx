import type { ReactNode } from "react";

interface Props {
  children: ReactNode;
}

export default function EmptyState({ children }: Props) {
  return (
    <div className="border border-dashed border-zinc-800 bg-black p-8 text-center text-zinc-400">
      {children}
    </div>
  );
}
