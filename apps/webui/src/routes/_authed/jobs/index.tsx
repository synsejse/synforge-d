import { createFileRoute } from "@tanstack/react-router";
import JobListPage from "../../../features/jobs/JobListPage";

export const Route = createFileRoute("/_authed/jobs/")({
  component: JobListPage,
});
