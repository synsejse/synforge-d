import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
}

export default class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("synforge ui error boundary:", error, info.componentStack);
  }

  render() {
    if (!this.state.error) {
      return this.props.children;
    }
    return (
      <div
        role="alert"
        aria-live="assertive"
        className="border-2 border-[var(--theme-error-red)] bg-black p-6"
      >
        <div className="font-mono text-xs font-bold uppercase tracking-[0.15em] text-[var(--theme-error-red)]">
          Page error
        </div>
        <p className="mt-3 text-sm text-zinc-200">
          Something went wrong rendering this page. Reload to try again.
        </p>
        <pre className="mt-4 overflow-x-auto border border-zinc-800 bg-zinc-950 p-3 font-mono text-xs text-zinc-400">
          {this.state.error.message}
        </pre>
      </div>
    );
  }
}
