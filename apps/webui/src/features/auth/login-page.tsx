import { useEffect, useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useRouter, useSearch } from "@tanstack/react-router";
import api, { ApiClientError } from "../../lib/api";
import { sessionQueries } from "../../lib/queries/session";

function safeNext(next: string | undefined): string {
  if (next && next.startsWith("/") && !next.startsWith("//")) {
    return next;
  }
  return "/";
}

export default function LoginPage() {
  const router = useRouter();
  const queryClient = useQueryClient();
  const search = useSearch({ from: "/login" });
  const [handle, setHandle] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      const status = await queryClient
        .ensureQueryData(sessionQueries.setupStatus())
        .catch(() => null);
      if (cancelled) return;
      if (status && !status.initialized) {
        router.navigate({ to: "/setup" });
        return;
      }
      const session = await queryClient
        .ensureQueryData(sessionQueries.current())
        .catch(() => null);
      if (cancelled) return;
      if (session) {
        router.navigate({ to: safeNext(search.next) });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [queryClient, router, search.next]);

  const loginMutation = useMutation({
    mutationFn: () => api.login({ handle: handle.trim(), password }),
    onSuccess: (session) => {
      queryClient.setQueryData(sessionQueries.current().queryKey, session);
      router.navigate({ to: safeNext(search.next) });
    },
    onError: (err) => {
      setError(err instanceof ApiClientError ? err.error.message : "Login failed.");
    },
  });

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    if (!handle.trim() || !password) {
      setError("Handle and password are required.");
      return;
    }
    loginMutation.mutate();
  };

  return (
    <div className="flex min-h-full items-center justify-center px-3 py-6 sm:px-6 sm:py-12">
      <div className="w-full max-w-md border-4 border-white bg-black p-4 shadow-[6px_6px_0_rgba(255,255,255,0.25)] sm:p-8">
        <div className="mb-8">
          <p className="font-mono text-xs font-bold uppercase tracking-[0.3em] text-[var(--theme-accent-lime)]">
            Synforge
          </p>
          <h1 className="mt-3 font-mono text-3xl font-bold uppercase text-zinc-50">
            Authorize WebUI
          </h1>
          <p className="mt-3 text-sm leading-6 text-zinc-400">
            {search.message ??
              "Enter a valid account handle and password to manage packages, builds, and logs."}
          </p>
        </div>

        <form className="space-y-4" onSubmit={onSubmit}>
          <label className="block">
            <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.16em] text-zinc-300">
              Handle
            </span>
            <input
              type="text"
              autoComplete="username"
              placeholder="admin"
              value={handle}
              onChange={(e) => setHandle(e.target.value)}
              className="w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-zinc-100 outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
            />
          </label>
          <label className="block">
            <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.16em] text-zinc-300">
              Password
            </span>
            <input
              type="password"
              autoComplete="current-password"
              placeholder="Password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full border-2 border-zinc-700 bg-black px-4 py-3 font-mono text-sm text-zinc-100 outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
            />
          </label>
          {error ? (
            <p className="border-2 border-[var(--theme-error-red)] bg-black px-3 py-2 text-sm text-zinc-200">
              {error}
            </p>
          ) : null}
          <button
            type="submit"
            disabled={loginMutation.isPending}
            className="w-full border-2 border-[var(--theme-accent-lime)] bg-[var(--theme-accent-lime)] px-4 py-3 font-mono text-xs font-bold uppercase tracking-[0.16em] text-black transition duration-100 ease-linear hover:-translate-x-[1px] hover:-translate-y-[1px] hover:bg-[#d8ff72] disabled:opacity-60"
          >
            {loginMutation.isPending ? "Signing in…" : "Enter Console"}
          </button>
        </form>
      </div>
    </div>
  );
}
