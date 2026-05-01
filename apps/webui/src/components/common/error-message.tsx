interface ErrorMessageProps {
  message: string;
}

export default function ErrorMessage({ message }: ErrorMessageProps) {
  return (
    <div role="alert" className="border-2 border-error bg-black p-4">
      <span className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-error">Error</span>
      <p className="mt-2 text-sm text-strong">{message}</p>
    </div>
  );
}
