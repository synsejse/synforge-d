import { createFileRoute } from "@tanstack/react-router";
import LoginPage from "../features/auth/LoginPage";

interface LoginSearch {
  next?: string;
  message?: string;
}

export const Route = createFileRoute("/login")({
  validateSearch: (search: Record<string, unknown>): LoginSearch => ({
    next: typeof search.next === "string" ? search.next : undefined,
    message: typeof search.message === "string" ? search.message : undefined,
  }),
  component: LoginPage,
});
