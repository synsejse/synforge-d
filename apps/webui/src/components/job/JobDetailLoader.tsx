import JobDetail from "./JobDetail";

export default function JobDetailLoader() {
  const jobId = typeof window !== "undefined"
    ? new URL(window.location.href).searchParams.get("id") ||
      window.location.pathname.split("/").filter(Boolean).at(-1) ||
      ""
    : "";
  
  if (!jobId) {
    return <div className="text-red-500">Invalid job ID</div>;
  }
  
  return <JobDetail jobId={jobId} />;
}
