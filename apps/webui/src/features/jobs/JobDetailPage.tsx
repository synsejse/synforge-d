import { useSearch } from "@tanstack/react-router";
import JobDetail from "./components/JobDetail";
import PageVisibilityProvider from "../../components/common/PageVisibilityProvider";

export default function JobDetailPage() {
  const search = useSearch({ from: "/_authed/jobs/view" });
  const jobId = search.id;

  if (!jobId) {
    return (
      <div className="flex min-h-[400px] items-center justify-center">
        <div className="font-mono text-sm text-zinc-500">
          No job ID provided
        </div>
      </div>
    );
  }

  return (
    <PageVisibilityProvider>
      <JobDetail jobId={jobId} />
    </PageVisibilityProvider>
  );
}
