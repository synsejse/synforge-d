import { createFileRoute } from "@tanstack/react-router";
import JobDetailPage from "../../../features/jobs/JobDetailPage";

interface JobViewSearch {
  id?: string;
}

export const Route = createFileRoute("/_authed/jobs/view")({
  validateSearch: (search: Record<string, unknown>): JobViewSearch => ({
    id: typeof search.id === "string" ? search.id : undefined,
  }),
  component: JobDetailPage,
});
