import { useEffect, useState, type FormEvent } from "react";
import { faBoxesStacked } from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { formatBytes } from "../../lib/bytes";
import type { PublishedRepoFile, RepoSummaryResponse } from "../../lib/types";
import ErrorMessage from "../common/ErrorMessage";
import PaginationControls from "../common/PaginationControls";
import LoadingBlock from "../ui/LoadingBlock";
import MetricCard from "../ui/MetricCard";
import PageHeader from "../ui/PageHeader";
import RepositoryInventoryFilters, {
  type RepoKindFilter,
} from "../repository/RepositoryInventoryFilters";
import RepositoryInventoryTable from "../repository/RepositoryInventoryTable";
import RepositoryRecentFilesSection from "../repository/RepositoryRecentFilesSection";
import RepositoryTargetsSection from "../repository/RepositoryTargetsSection";

const PAGE_SIZE = 50;

export default function RepositoryBrowser() {
  const [summary, setSummary] = useState<RepoSummaryResponse | null>(null);
  const [files, setFiles] = useState<PublishedRepoFile[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [offset, setOffset] = useState(() => {
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
  const [kindFilter, setKindFilter] = useState<RepoKindFilter>(() => {
    if (typeof window === "undefined") {
      return "all";
    }
    const value = new URLSearchParams(window.location.search).get("kind");
    return value === "rpm" || value === "srpm" || value === "log"
      ? value
      : "all";
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function load(
    nextOffset = offset,
    nextPackageFilter = packageFilter,
    nextTargetFilter = targetFilter,
    nextKindFilter = kindFilter,
  ) {
    try {
      setLoading(true);
      const [summaryRes, inventoryRes] = await Promise.all([
        api.getRepoSummary(),
        api.getRepoInventory(PAGE_SIZE, nextOffset, {
          packageName: nextPackageFilter,
          mockChroot: nextTargetFilter,
          kind: nextKindFilter,
        }),
      ]);
      setSummary(summaryRes);
      setFiles(inventoryRes.repo_files);
      setHasMore(inventoryRes.page.has_more);
      setOffset(nextOffset);
      setPackageFilter(nextPackageFilter);
      setTargetFilter(nextTargetFilter);
      setKindFilter(nextKindFilter);
      setError(null);
      if (typeof window !== "undefined") {
        const params = new URLSearchParams();
        if (nextOffset > 0) {
          params.set("offset", String(nextOffset));
        }
        if (nextPackageFilter.trim()) {
          params.set("package", nextPackageFilter.trim());
        }
        if (nextTargetFilter.trim()) {
          params.set("target", nextTargetFilter.trim());
        }
        if (nextKindFilter !== "all") {
          params.set("kind", nextKindFilter);
        }
        const query = params.toString();
        window.history.replaceState({}, "", `/repository/${query ? `?${query}` : ""}`);
      }
    } catch (e) {
      setError(
        e instanceof Error ? e.message : "Failed to load repository inventory",
      );
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  function handleApply(event: FormEvent) {
    event.preventDefault();
    load(0, packageFilter, targetFilter, kindFilter);
  }

  if (loading && !summary) {
    return <LoadingBlock label="Loading repository inventory…" lines={4} />;
  }

  if (error) {
    return <ErrorMessage message={error} />;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Managed Repository"
        title="Repository Control"
        description="Published packages, builds, and files."
        actions={[{ href: "/packages/", label: "Packages", icon: faBoxesStacked }]}
      />

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          label="Packages"
          value={summary?.package_count ?? 0}
          detail="Published package names"
        />
        <MetricCard
          label="Targets"
          value={summary?.target_count ?? 0}
          detail="Active build targets"
        />
        <MetricCard
          label="Builds"
          value={summary?.build_count ?? 0}
          detail="Recorded publish jobs"
        />
        <MetricCard
          label="Stored Size"
          value={formatBytes(summary?.stored_bytes ?? 0)}
          detail={`${summary?.published_file_count ?? 0} published files`}
        />
      </section>

      <RepositoryTargetsSection targets={summary?.targets ?? []} />
      <RepositoryRecentFilesSection recentFiles={summary?.recent_files ?? []} />

      <section className="space-y-4">
        <RepositoryInventoryFilters
          packageFilter={packageFilter}
          targetFilter={targetFilter}
          kindFilter={kindFilter}
          onPackageFilterChange={setPackageFilter}
          onTargetFilterChange={setTargetFilter}
          onKindFilterChange={setKindFilter}
          onApply={handleApply}
        />

        <div className="flex items-end justify-between gap-4">
          <div>
            <div className="text-xs uppercase tracking-[0.24em] text-zinc-500">
              Inventory
            </div>
            <h2 className="mt-2 text-2xl font-semibold text-white">
              Published files
            </h2>
          </div>
          <div className="text-sm text-zinc-500">Server-paginated inventory view</div>
        </div>

        <RepositoryInventoryTable files={files} />

        {files.length > 0 ? (
          <PaginationControls
            onPrevious={() => load(Math.max(0, offset - PAGE_SIZE))}
            onNext={() => load(offset + PAGE_SIZE)}
            previousDisabled={loading || offset === 0}
            nextDisabled={loading || !hasMore}
          />
        ) : null}
      </section>
    </div>
  );
}
