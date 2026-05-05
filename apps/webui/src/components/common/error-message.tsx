interface ErrorMessageProps {
  message: string;
}

export default function ErrorMessage({ message }: ErrorMessageProps) {
  return (
    <div role="alert" className="border-2 border-error bg-black p-4">
      <div className="flex items-center gap-2">
        <span className="synforge-glitch-once font-mono text-xs font-bold uppercase tracking-[0.18em] text-error">
          [ERR]
        </span>
        <span
          aria-hidden="true"
          className="h-px flex-1 bg-error/40"
        />
      </div>
      <p className="mt-2 text-sm text-strong">{message}</p>
    </div>
  );
}
