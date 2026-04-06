import { useEffect, useState } from "react";
import JobDetail from "./JobDetail";

export default function JobDetailLoader() {
  const [jobId, setJobId] = useState<string | null>(null);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const params = new URLSearchParams(window.location.search);
    const id = params.get("id");
    if (id) {
      setJobId(id);
    }
  }, []);

  if (!jobId) {
    return (
      <div className="flex min-h-[400px] items-center justify-center">
        <div className="font-mono text-sm text-zinc-500">
          No job ID provided
        </div>
      </div>
    );
  }

  return <JobDetail jobId={jobId} />;
}
