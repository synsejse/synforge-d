import { createFileRoute } from "@tanstack/react-router";
import RepositoryBrowserPage from "../../../features/repository/RepositoryBrowserPage";

export const Route = createFileRoute("/_authed/repository/")({
  component: RepositoryBrowserPage,
});
