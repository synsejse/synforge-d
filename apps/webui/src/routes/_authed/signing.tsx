import { createFileRoute } from "@tanstack/react-router";
import SigningPage from "../../features/signing/SigningPage";

export const Route = createFileRoute("/_authed/signing")({
  component: SigningPage,
});
