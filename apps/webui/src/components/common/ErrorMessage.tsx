interface ErrorMessageProps {
  message: string;
}

export default function ErrorMessage({ message }: ErrorMessageProps) {
  return (
    <div role="alert" className="border-2 border-[var(--theme-error-red)] bg-black p-4">
      <span className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-[var(--theme-error-red)]">Error</span>
      <p className="mt-2 text-sm text-zinc-200">{message}</p>
    </div>
  );
}
