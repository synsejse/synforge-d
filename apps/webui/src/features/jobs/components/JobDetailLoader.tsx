import { useEffect, useState } from "react";
import JobDetail from "./JobDetail";
import PageRoot from "../../../components/common/PageRoot";
import PageVisibilityProvider from "../../../components/common/PageVisibilityProvider";
import ServerHardwareProvider from "../../../components/common/ServerHardwareProvider";

function JobDetailLoaderInner() {
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

export default function JobDetailLoader() {
  return (
    <PageRoot>
      <PageVisibilityProvider>
        <ServerHardwareProvider>
          <JobDetailLoaderInner />
        </ServerHardwareProvider>
      </PageVisibilityProvider>
    </PageRoot>
  );
}
