import { createFileRoute } from "@tanstack/react-router";
import RepositorySetupPage from "../../../features/repository/RepositorySetupPage";

export const Route = createFileRoute("/_authed/repository/use")({
  component: RepositorySetupPage,
});
