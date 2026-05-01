import { createFileRoute } from "@tanstack/react-router";
import JobDetail from "../../../features/jobs/components/job-detail";

interface JobViewSearch {
  id?: string;
}

export const Route = createFileRoute("/_authed/jobs/view")({
  validateSearch: (search: Record<string, unknown>): JobViewSearch => ({
    id: typeof search.id === "string" ? search.id : undefined,
  }),
  component: JobView,
});

function JobView() {
  const { id } = Route.useSearch();
  if (!id) {
    return (
      <div className="flex min-h-[400px] items-center justify-center">
        <div className="font-mono text-sm text-soft">
          No job ID provided
        </div>
      </div>
    );
  }
  return <JobDetail jobId={id} />;
}
