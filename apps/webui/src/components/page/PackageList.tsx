import { useEffect, useState } from "react";
import {
  faPlus,
  faRotate,
  faHammer,
  faTrash,
  faFolderOpen,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { summarizePackageAction } from "../../lib/package-actions";
import { formatDurationSeconds } from "../../lib/datetime";
import type {
  PackageTargetState,
  PackageResponse,
  RefreshAllPackagesProgressView,
} from "../../lib/types";
import AddPackageModal from "../package/AddPackageModal";
import ErrorMessage from "../common/ErrorMessage";
import LoadingBlock from "../ui/LoadingBlock";
import FaIcon from "../ui/FaIcon";
import Badge from "../ui/Badge";
import Button from "../ui/Button";
import Select from "../ui/Select";
import PageHeader from "../ui/PageHeader";
import ProgressOverlayDialog from "../ui/ProgressOverlayDialog";

export default function PackageList() {
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
      <div className="border-2 border-white bg-black p-5">
        <div className="grid gap-4 md:grid-cols-[1fr_auto_auto]">
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
          <div className="flex items-end">
            <Button
              variant="secondary"
              size="md"
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
        <div className="flex items-center justify-between border-2 border-white bg-black p-4">
          <Button
            variant="secondary"
            size="md"
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

interface PackageCardProps {
  entry: PackageResponse;
  onRefresh: (name: string) => void;
  onRebuild: (name: string) => void;
  onDelete: (name: string) => void;
  refreshing: boolean;
  refreshDisabled: boolean;
}

function PackageCard({
  entry,
  onRefresh,
  onRebuild,
  onDelete,
  refreshing,
  refreshDisabled,
}: PackageCardProps) {
  const pkg = entry.package;
  const backoffTargets = entry.state.targets.filter(isBackoffActive);
  const backoffSummary =
    backoffTargets.length > 0 ? summarizeBackoffTargets(backoffTargets) : null;
  const status = pkg.enabled
    ? entry.builds_pending || entry.builds_running
      ? "running"
      : "success"
    : "disabled";

  return (
    <article className="border-2 border-white bg-black transition duration-100 ease-linear hover:-translate-x-[2px] hover:-translate-y-[2px] hover:shadow-[4px_4px_0_rgba(255,255,255,0.3)]">
      <div className="border-b-2 border-zinc-800 bg-black px-6 py-5">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div className="min-w-0 flex-1">
            <a
              href={`/packages/view/?name=${encodeURIComponent(pkg.name)}`}
              className="font-mono text-lg font-bold uppercase text-white transition duration-100 ease-linear hover:text-[var(--theme-accent-lime)]"
            >
              {pkg.name}
            </a>
            <p className="mt-1 text-sm text-zinc-500">
              {pkg.description || "No description"}
            </p>
          </div>
          <div className="flex items-center gap-2">
            {backoffSummary && (
              <Badge variant="warning" title={backoffSummary.details}>
                BACKOFF {backoffSummary.count}
              </Badge>
            )}
            <Badge 
              variant={status === "running" ? "lime" : status === "success" ? "terminal-green" : "default"} 
              pulse={status === "running"}
            >
              {status === "running" ? "ACTIVE" : status === "success" ? "READY" : "DISABLED"}
            </Badge>
          </div>
        </div>
      </div>

      {backoffSummary && (
        <div className="border-t-2 border-zinc-800 bg-zinc-950 px-6 py-3 text-xs text-zinc-300">
          <span className="font-mono uppercase tracking-[0.14em] text-[var(--theme-accent-orange)]">
            Failure backoff active:
          </span>{" "}
          <span className="font-mono">{backoffSummary.details}</span>
        </div>
      )}

      <div className="grid gap-px bg-zinc-800 md:grid-cols-2">
        <div className="bg-black px-5 py-4">
          <div className="font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-zinc-600">
            VERSION
          </div>
          <div className="mt-2 font-mono text-sm text-white">
            {pkg.version}-{pkg.release}
          </div>
        </div>
        <div className="bg-black px-5 py-4">
          <div className="font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-zinc-600">
            TARGETS
          </div>
          <div className="mt-2 font-mono text-sm text-zinc-300">
            {pkg.mock_chroots.join(", ") || "None"}
          </div>
        </div>
        <div className="bg-black px-5 py-4 md:col-span-2">
          <div className="font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-zinc-600">
            REPOSITORY
          </div>
          <div className="mt-2 break-all font-mono text-sm text-zinc-400">
            {pkg.source.repo_url}
          </div>
        </div>
      </div>

      <div className="flex flex-wrap gap-2 border-t-2 border-zinc-800 bg-black px-6 py-4">
        <Button
          type="button"
          variant="secondary"
          size="sm"
          onClick={() => onRefresh(pkg.name)}
          disabled={refreshDisabled || refreshing}
          aria-busy={refreshing || undefined}
        >
          <FaIcon icon={faRotate} className={refreshing ? "mr-2 animate-spin" : "mr-2"} />
          {refreshing ? "Refreshing…" : "Refresh"}
        </Button>
        <Button variant="secondary" size="sm" onClick={() => onRebuild(pkg.name)}>
          <FaIcon icon={faHammer} className="mr-2" />
          Rebuild
        </Button>
        <a href={`/packages/view/?name=${encodeURIComponent(pkg.name)}`}>
          <Button variant="ghost" size="sm">
            <FaIcon icon={faFolderOpen} className="mr-2" />
            Details
          </Button>
        </a>
        <Button variant="danger" size="sm" onClick={() => onDelete(pkg.name)}>
          <FaIcon icon={faTrash} className="mr-2" />
          Delete
        </Button>
      </div>
    </article>
  );
}

function isBackoffActive(target: PackageTargetState): boolean {
  const remaining = target.backoff_remaining_seconds ?? null;
  return remaining !== null && remaining > 0;
}

function summarizeBackoffTargets(targets: PackageTargetState[]): {
  count: number;
  details: string;
} {
  const details = targets
    .map((target) => {
      const remaining = target.backoff_remaining_seconds ?? 0;
      return `${target.mock_chroot} (${formatDurationSeconds(remaining)})`;
    })
    .join(", ");

  return { count: targets.length, details };
}
