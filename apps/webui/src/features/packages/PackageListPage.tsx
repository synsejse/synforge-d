import { useEffect, useState } from "react";
import {
  faPlus,
  faRotate,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { summarizePackageAction } from "../../lib/package-actions";
import type {
  PackageResponse,
  RefreshAllPackagesProgressView,
} from "../../lib/types";
import AddPackageModal from "./components/AddPackageModal";
import PackageCard from "./components/PackageCard";
import ErrorBoundary from "../../components/common/ErrorBoundary";
import ErrorMessage from "../../components/common/ErrorMessage";
import LoadingBlock from "../../components/ui/LoadingBlock";
import Button from "../../components/ui/Button";
import Select from "../../components/ui/Select";
import PageHeader from "../../components/ui/PageHeader";
import ProgressOverlayDialog from "../../components/ui/ProgressOverlayDialog";

function PackageList() {
  const [packages, setPackages] = useState<PackageResponse[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [offset, setOffset] = useState(() => {
    if (typeof window === "undefined") {
      return 0;
    }
    return Number(new URLSearchParams(window.location.search).get("offset") || "0");
  });
  const [search, setSearch] = useState(() => {
    if (typeof window === "undefined") {
      return "";
    }
    return new URLSearchParams(window.location.search).get("search") || "";
  });
  const [enabledFilter, setEnabledFilter] = useState<"all" | "true" | "false">(() => {
    if (typeof window === "undefined") {
      return "all";
    }
    const value = new URLSearchParams(window.location.search).get("enabled");
    return value === "true" || value === "false" ? value : "all";
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);
  const [refreshingAll, setRefreshingAll] = useState(false);
  const [refreshingPackageNames, setRefreshingPackageNames] = useState<Set<string>>(
    () => new Set(),
  );
  const [refreshOverlayOpen, setRefreshOverlayOpen] = useState(false);
  const [refreshOverlayTitle, setRefreshOverlayTitle] = useState("Refreshing packages");
  const [refreshOverlayDetail, setRefreshOverlayDetail] = useState(
    "Preparing package refresh…",
  );
  const [refreshOverlayProgress, setRefreshOverlayProgress] = useState(0);
  const pageSize = 50;

  async function load(
    nextOffset = offset,
    nextSearch = search,
    nextEnabled = enabledFilter,
  ) {
    try {
      setLoading(true);
      const res = await api.listPackagesPage(pageSize, nextOffset, {
        search: nextSearch,
        enabled: nextEnabled === "all" ? "all" : nextEnabled === "true",
      });
      setPackages(res.packages);
      setHasMore(res.page.has_more);
      setOffset(nextOffset);
      setSearch(nextSearch);
      setEnabledFilter(nextEnabled);
      setError(null);

      if (typeof window !== "undefined") {
        const params = new URLSearchParams();
        if (nextOffset > 0) {
          params.set("offset", String(nextOffset));
        }
        if (nextSearch.trim()) {
          params.set("search", nextSearch.trim());
        }
        if (nextEnabled !== "all") {
          params.set("enabled", nextEnabled);
        }
        const query = params.toString();
        window.history.replaceState({}, "", `/packages/${query ? `?${query}` : ""}`);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load packages");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  async function handleDelete(name: string) {
    if (!confirm(`Delete package "${name}"?`)) return;
    try {
      await api.deletePackage(name);
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : "Failed to delete package");
    }
  }

  async function trigger(name: string, action: "refresh" | "rebuild") {
    const isRefresh = action === "refresh";
    if (isRefresh && refreshingPackageNames.has(name)) {
      return;
    }
    if (isRefresh) {
      setRefreshingPackageNames((current) => {
        const next = new Set(current);
        next.add(name);
        return next;
      });
    }
    try {
      const response =
        isRefresh
          ? await api.refreshPackage(name)
          : await api.rebuildPackage(name);
      alert(summarizePackageAction(response));
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : `Failed to ${action} package`);
    } finally {
      if (isRefresh) {
        setRefreshingPackageNames((current) => {
          if (!current.has(name)) {
            return current;
          }
          const next = new Set(current);
          next.delete(name);
          return next;
        });
      }
    }
  }

  function applyRefreshAllProgress(operation: RefreshAllPackagesProgressView) {
    if (operation.state === "running") {
      setRefreshOverlayTitle("Refreshing enabled packages");
    } else if (operation.state === "failed") {
      setRefreshOverlayTitle("Refresh all failed");
    } else {
      setRefreshOverlayTitle("Refresh all complete");
    }

    if (operation.total_packages === 0) {
      setRefreshOverlayProgress(operation.state === "running" ? 0 : 100);
      setRefreshOverlayDetail(operation.message ?? "Preparing package refresh…");
      return;
    }

    const progressPercent = Math.min(
      100,
      Math.round((operation.processed_packages / operation.total_packages) * 100),
    );
    setRefreshOverlayProgress(progressPercent);
    const detail = [
      `${operation.processed_packages}/${operation.total_packages} packages`,
      `queued ${operation.queued_packages}`,
      `skipped ${operation.skipped_packages}`,
      `blocked ${operation.blocked_packages}`,
      `failed ${operation.failed_packages}`,
      `queued targets ${operation.queued_targets}`,
      `skipped targets ${operation.skipped_targets}`,
      `blocked targets ${operation.blocked_targets}`,
    ].join(" · ");
    setRefreshOverlayDetail(
      operation.message ? `${detail} · ${operation.message}` : detail,
    );
  }

  async function pollRefreshAllProgress() {
    const progress = await api.getRefreshAllPackagesProgress();
    if (!progress.operation) {
      return;
    }
    applyRefreshAllProgress(progress.operation);
  }

  async function handleRefreshAllPackages() {
    if (refreshingAll) {
      return;
    }
    if (!confirm("Queue manual refresh for all enabled packages?")) {
      return;
    }

    let progressTicker: number | null = null;
    try {
      setRefreshingAll(true);
      setError(null);
      setRefreshOverlayOpen(true);
      setRefreshOverlayProgress(0);
      setRefreshOverlayTitle("Refreshing enabled packages");
      setRefreshOverlayDetail("Preparing package refresh…");
      progressTicker = window.setInterval(() => {
        void pollRefreshAllProgress().catch(() => undefined);
      }, 500);
      await pollRefreshAllProgress().catch(() => undefined);
      const response = await api.refreshAllPackages();
      applyRefreshAllProgress(response.operation);
      await pollRefreshAllProgress().catch(() => undefined);
      await load();
    } catch (e) {
      const message =
        e instanceof Error ? e.message : "Failed to refresh enabled packages";
      setRefreshOverlayTitle("Refresh all failed");
      setRefreshOverlayDetail(message);
    } finally {
      if (progressTicker !== null) {
        window.clearInterval(progressTicker);
      }
      setRefreshingAll(false);
    }
  }

  if (loading) {
    return <LoadingBlock label="Loading packages…" lines={4} />;
  }

  if (error) {
    return <ErrorMessage message={error} />;
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <PageHeader
        eyebrow="PACKAGE_REGISTRY"
        title="Packages"
        description="Sources, targets, and builds."
        color="lime"
        actions={[
          {
            onClick: () => void handleRefreshAllPackages(),
            label: refreshingAll ? "Refreshing…" : "Refresh All",
            icon: faRotate,
          },
          {
            onClick: () => setShowAddModal(true),
            label: "Add Package",
            icon: faPlus,
            variant: "primary",
          },
        ]}
      />

      {/* Filters */}
      <div className="border-2 border-white bg-black p-4 sm:p-5">
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-[minmax(0,1fr)_220px_auto]">
          <div>
            <label className="block">
              <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-400">
                Search
              </span>
              <input
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && load(0, search, enabledFilter)}
                placeholder="Filter by name or description"
                className="w-full border-2 border-zinc-700 bg-black px-4 py-2.5 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
              />
            </label>
          </div>
          <div>
            <label className="block">
              <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-400">
                Status
              </span>
              <Select
                value={enabledFilter}
                onValueChange={(val) => setEnabledFilter(val as "all" | "true" | "false")}
                options={[
                  { value: "all", label: "All" },
                  { value: "true", label: "Enabled" },
                  { value: "false", label: "Disabled" },
                ]}
              />
            </label>
          </div>
          <div className="flex items-end md:col-span-2 xl:col-span-1">
            <Button
              variant="secondary"
              size="md"
              className="w-full xl:w-auto"
              onClick={() => load(0, search, enabledFilter)}
            >
              Apply Filters
            </Button>
          </div>
        </div>
      </div>

      {/* Package Cards */}
      {packages.length === 0 ? (
        <div className="border-2 border-zinc-700 bg-black p-12 text-center">
          <p className="font-mono text-sm font-bold uppercase tracking-[0.3em] text-zinc-500">
            NO_PACKAGES_CONFIGURED
          </p>
          <p className="mt-2 text-sm text-zinc-600">
            Add a spec source to start building.
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {packages.map((entry) => (
            <PackageCard
              key={entry.package.name}
              entry={entry}
              onRefresh={(name) => void trigger(name, "refresh")}
              onRebuild={(name) => void trigger(name, "rebuild")}
              onDelete={(name) => void handleDelete(name)}
              refreshing={refreshingPackageNames.has(entry.package.name)}
              refreshDisabled={refreshingAll}
            />
          ))}
        </div>
      )}

      {/* Pagination */}
      {packages.length > 0 && (
        <div className="flex flex-col gap-3 border-2 border-white bg-black p-4 sm:flex-row sm:items-center sm:justify-between">
          <Button
            variant="secondary"
            size="md"
            className="w-full sm:w-auto"
            onClick={() => load(Math.max(0, offset - pageSize), search, enabledFilter)}
            disabled={loading || offset === 0}
          >
            Previous
          </Button>
          <span className="font-mono text-sm text-zinc-400">
            Offset: {offset}
          </span>
          <Button
            variant="secondary"
            size="md"
            className="w-full sm:w-auto"
            onClick={() => load(offset + pageSize, search, enabledFilter)}
            disabled={loading || !hasMore}
          >
            Next
          </Button>
        </div>
      )}

      {showAddModal && (
        <AddPackageModal
          onClose={() => setShowAddModal(false)}
          onSuccess={() => {
            setShowAddModal(false);
            load();
          }}
        />
      )}

      <ProgressOverlayDialog
        open={refreshOverlayOpen}
        title={refreshOverlayTitle}
        detail={refreshOverlayDetail}
        progress={refreshOverlayProgress}
        onClose={() => setRefreshOverlayOpen(false)}
        closeDisabled={refreshingAll}
      />
    </div>
  );
}

export default function PackageListPage() {
  return (
    <ErrorBoundary>
      <PackageList />
    </ErrorBoundary>
  );
}
