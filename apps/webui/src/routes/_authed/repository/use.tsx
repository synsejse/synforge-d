import { createFileRoute } from "@tanstack/react-router";
import RepositorySetupPage from "../../../features/repository/repository-setup-page";

export const Route = createFileRoute("/_authed/repository/use")({
  component: RepositorySetupPage,
});
