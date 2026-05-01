export default function MockTargetCheckIndicator({ label }: { label: string }) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between gap-3">
        <span className="font-mono text-xs uppercase tracking-[0.15em] text-muted">
          {label}
        </span>
        <span className="flex items-center gap-1" aria-hidden="true">
          <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-accent-lime" />
          <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-accent-lime [animation-delay:150ms]" />
          <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-accent-lime [animation-delay:300ms]" />
        </span>
      </div>
      <div className="h-2 w-full overflow-hidden border border-edge-strong bg-surface-hover">
        <div className="h-full w-full animate-pulse bg-accent-lime/65" />
      </div>
    </div>
  );
}
