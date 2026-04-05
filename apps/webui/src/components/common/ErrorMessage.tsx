interface ErrorMessageProps {
  message: string;
}

export default function ErrorMessage({ message }: ErrorMessageProps) {
  return (
    <div role="alert" className="border border-zinc-800 bg-black p-4 text-zinc-200">
      Error: {message}
    </div>
  );
}
