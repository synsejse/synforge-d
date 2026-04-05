import { useEffect, useState } from "react";
import { faChartLine } from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import type { BuildJobResponse } from "../../lib/types";
import ErrorMessage from "../common/ErrorMessage";
import PaginationControls from "../common/PaginationControls";
import EmptyState from "../ui/EmptyState";
import JobModeFilterBar, { type JobViewMode } from "../job/JobModeFilterBar";
import JobSearchFilters from "../job/JobSearchFilters";
import JobTable from "../job/JobTable";
import LoadingBlock from "../ui/LoadingBlock";
import PageHeader from "../ui/PageHeader";

export default function JobList() {
  const [jobs, setJobs] = useState<BuildJobResponse[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [mode, setMode] = useState<JobViewMode>(() => {
    if (typeof window === "undefined") {
      return "history";
    }
    const value = new URLSearchParams(window.location.search).get("mode");
    return value === "active" ? "active" : "history";
  });
  const [filter, setFilter] = useState<string>(() => {
    if (typeof window === "undefined") {
      return "all";
    }
    return new URLSearchParams(window.location.search).get("status") || "all";
  });
  const [offset, setOffset] = useState<number>(() => {
    if (typeof window === "undefined") {
      return 0;
    }
    return Number(new URLSearchParams(window.location.search).get("offset") || "0");
  });
  const [packageFilter, setPackageFilter] = useState(() => {
    if (typeof window === "undefined") {
      return "";
    }
    return new URLSearchParams(window.location.search).get("package") || "";
  });
  const [targetFilter, setTargetFilter] = useState(() => {
    if (typeof window === "undefined") {
      return "";
    }
    return new URLSearchParams(window.location.search).get("target") || "";
  });
  const [pruning, setPruning] = useState(false);
  const pageSize = 50;

  async function load(
    nextMode = mode,
    nextFilter = filter,
    nextOffset = offset,
    nextPackageFilter = packageFilter,
    nextTargetFilter = targetFilter,
  ) {
    try {
      setLoading(true);
      const res =
        nextMode === "active"
          ? await api.listActiveJobs({
              limit: pageSize,
              offset: nextOffset,
              packageName: nextPackageFilter,
              mockChroot: nextTargetFilter,
            })
          : await api.listCompletedJobs({
              limit: pageSize,
              offset: nextOffset,
              status: nextFilter,
              packageName: nextPackageFilter,
              mockChroot: nextTargetFilter,
            });
      setJobs(res.jobs);
      setHasMore(res.page.has_more);
      setMode(nextMode);
      setOffset(nextOffset);
      setFilter(nextFilter);
      setPackageFilter(nextPackageFilter);
      setTargetFilter(nextTargetFilter);
      setError(null);
      if (typeof window !== "undefined") {
        const params = new URLSearchParams(window.location.search);
        if (nextMode === "history") {
          params.delete("mode");
        } else {
          params.set("mode", nextMode);
        }
        if (nextMode === "active" || nextFilter === "all") {
          params.delete("status");
        } else {
          params.set("status", nextFilter);
        }
        if (nextOffset === 0) {
          params.delete("offset");
        } else {
          params.set("offset", String(nextOffset));
        }
        if (nextPackageFilter.trim()) {
          params.set("package", nextPackageFilter.trim());
        } else {
          params.delete("package");
        }
        if (nextTargetFilter.trim()) {
          params.set("target", nextTargetFilter.trim());
        } else {
          params.delete("target");
        }
        const query = params.toString();
        window.history.replaceState({}, "", `/jobs/${query ? `?${query}` : ""}`);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load jobs");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  async function handleDelete(job: BuildJobResponse) {
    if (!confirm(`Delete job ${job.job.id}?`)) {
      return;
    }
    try {
      await api.deleteJob(job.job.id);
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : "Failed to delete job");
    }
  }

  async function handlePruneFailed() {
    const failedCount = jobs.filter(
      (entry) =>
        entry.job.status === "failed" || entry.job.status === "timed_out",
    ).length;
    if (failedCount === 0) {
      return;
    }
    if (!confirm(`Delete ${failedCount} failed or timed out jobs?`)) {
      return;
    }
    try {
      setPruning(true);
      await api.pruneFailedJobs();
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : "Failed to prune failed jobs");
    } finally {
      setPruning(false);
    }
  }

  if (loading) {
    return <LoadingBlock label="Loading jobs…" lines={4} />;
  }

  if (error) {
    return <ErrorMessage message={error} />;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Job Activity"
        title={mode === "active" ? "Active Builds" : "Build Timeline"}
        description={
          mode === "active"
            ? "Pending and running jobs currently in flight."
            : "Finished job history across all packages and targets."
        }
        actions={[{ href: "/", label: "Overview", icon: faChartLine }]}
      />

      <JobModeFilterBar
        mode={mode}
        filter={filter}
        pruning={pruning}
        jobs={jobs}
        onModeChange={(nextMode) => load(nextMode, "all", 0, packageFilter, targetFilter)}
        onFilterChange={(nextFilter) =>
          load(mode, nextFilter, 0, packageFilter, targetFilter)
        }
        onPruneFailed={handlePruneFailed}
      />

      <JobSearchFilters
        packageFilter={packageFilter}
        targetFilter={targetFilter}
        onPackageFilterChange={setPackageFilter}
        onTargetFilterChange={setTargetFilter}
        onApply={() => load(mode, filter, 0, packageFilter, targetFilter)}
      />

      {jobs.length === 0 ? (
        <EmptyState>
          {mode === "active"
            ? "No active jobs match the current filter."
            : "No finished jobs match the current filter."}
        </EmptyState>
      ) : (
        <JobTable jobs={jobs} onDelete={handleDelete} />
      )}

      {jobs.length > 0 ? (
        <PaginationControls
          onPrevious={() =>
            load(mode, filter, Math.max(0, offset - pageSize), packageFilter, targetFilter)
          }
          onNext={() => load(mode, filter, offset + pageSize, packageFilter, targetFilter)}
          previousDisabled={loading || offset === 0}
          nextDisabled={loading || !hasMore}
        />
      ) : null}
    </div>
  );
}
