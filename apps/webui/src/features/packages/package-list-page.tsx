import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
import {
  faPlus,
  faRotate,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { packagesQueries } from "../../lib/queries";

const route = getRouteApi("/_authed/packages/");
import { summarizePackageAction } from "../../lib/package-actions";
import type { RefreshAllPackagesProgressView } from "../../lib/types";
import AddPackageModal from "./components/add-package-modal";
import PackageCard from "./components/package-card";
import ErrorMessage from "../../components/common/error-message";
import { useDialogs } from "../../components/common/dialogs-provider";
import { useToast } from "../../components/common/toast-provider";
import LoadingBlock from "../../components/ui/loading-block";
import Button from "../../components/ui/button";
import Select from "../../components/ui/select";
import PageHeader from "../../components/ui/page-header";
import ProgressOverlayDialog from "../../components/ui/progress-overlay-dialog";
import PaginationControls from "../../components/common/pagination-controls";
import FilterBar from "../../components/common/filter-bar";

const PAGE_SIZE = 50;

type EnabledFilter = "all" | "true" | "false";

function formatRefreshDetail(operation: RefreshAllPackagesProgressView): string {
  if (operation.total_packages === 0) {
    return operation.message ?? "Preparing package refresh…";
  }
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
  return operation.message ? `${detail} · ${operation.message}` : detail;
}

function refreshTitle(state: RefreshAllPackagesProgressView["state"]): string {
  if (state === "running") return "Refreshing enabled packages";
  if (state === "failed") return "Refresh all failed";
  return "Refresh all complete";
}

function PackageList() {
  const queryClient = useQueryClient();
  const { confirm } = useDialogs();
  const toast = useToast();
  const navigate = route.useNavigate();
  const rawSearch = route.useSearch();
  const offset = rawSearch.offset ?? 0;
  const search = rawSearch.search ?? "";
  const enabledFilter = rawSearch.enabled ?? "all";
  const [searchInput, setSearchInput] = useState(search);
  const [showAddModal, setShowAddModal] = useState(false);
  const [refreshOverlayOpen, setRefreshOverlayOpen] = useState(false);

  const setOffset = (next: number) =>
    navigate({ search: (prev) => ({ ...prev, offset: next }) });
  const setSearch = (next: string) =>
    navigate({ search: (prev) => ({ ...prev, offset: 0, search: next }) });
  const setEnabledFilter = (next: EnabledFilter) =>
    navigate({ search: (prev) => ({ ...prev, offset: 0, enabled: next }) });

  const listQuery = useQuery(
    packagesQueries.list({
      limit: PAGE_SIZE,
      offset,
      search,
      enabled: enabledFilter === "all" ? "all" : enabledFilter === "true",
    }),
  );

  const invalidatePackages = () =>
    queryClient.invalidateQueries({ queryKey: ["packages"] });

  const deleteMutation = useMutation({
    mutationFn: (name: string) => api.deletePackage(name),
    onSuccess: invalidatePackages,
    onError: (error) =>
      toast.error(
        "Delete failed",
        error instanceof Error ? error.message : "Failed to delete package",
      ),
  });

  const triggerMutation = useMutation({
    mutationFn: ({ name, action }: { name: string; action: "refresh" | "rebuild" }) =>
      action === "refresh" ? api.refreshPackage(name) : api.rebuildPackage(name),
    onSuccess: (response, variables) => {
      toast.success(
        variables.action === "refresh" ? "Refresh queued" : "Rebuild queued",
        summarizePackageAction(response),
      );
      void invalidatePackages();
    },
    onError: (error, variables) =>
      toast.error(
        `${variables.action === "refresh" ? "Refresh" : "Rebuild"} failed`,
        error instanceof Error
          ? error.message
          : `Failed to ${variables.action} package`,
      ),
  });

  const refreshAllMutation = useMutation({
    mutationFn: () => api.refreshAllPackages(),
    onSettled: () => invalidatePackages(),
  });

  const progressQuery = useQuery({
    ...packagesQueries.refreshAllProgress(),
    enabled: refreshOverlayOpen,
    refetchInterval: refreshOverlayOpen ? 500 : false,
  });

  const refreshingAll = refreshAllMutation.isPending;
  const liveOperation =
    refreshAllMutation.data?.operation ?? progressQuery.data?.operation ?? null;

  async function handleDelete(name: string) {
    const ok = await confirm({
      title: "Delete package?",
      message: `Package "${name}" will be removed.`,
      confirmLabel: "Delete",
      destructive: true,
    });
    if (!ok) return;
    deleteMutation.mutate(name);
  }

  function trigger(name: string, action: "refresh" | "rebuild") {
    if (
      action === "refresh" &&
      triggerMutation.isPending &&
      triggerMutation.variables?.name === name &&
      triggerMutation.variables?.action === "refresh"
    ) {
      return;
    }
    triggerMutation.mutate({ name, action });
  }

  async function handleRefreshAllPackages() {
    if (refreshingAll) return;
    const ok = await confirm({
      title: "Refresh all enabled packages?",
      message: "Queue a manual source refresh for every enabled package.",
      confirmLabel: "Refresh all",
    });
    if (!ok) return;
    setRefreshOverlayOpen(true);
    await refreshAllMutation.mutateAsync().catch(() => undefined);
  }

  function applyFilters() {
    setSearch(searchInput);
  }

  if (listQuery.isPending) {
    return <LoadingBlock label="Loading packages…" lines={4} />;
  }

  if (listQuery.error) {
    return (
      <ErrorMessage
        message={
          listQuery.error instanceof Error
            ? listQuery.error.message
            : "Failed to load packages"
        }
      />
    );
  }

  const refreshingNameForMutation =
    triggerMutation.isPending &&
    triggerMutation.variables?.action === "refresh"
      ? triggerMutation.variables.name
      : null;

  const overlayTitle = liveOperation
    ? refreshTitle(liveOperation.state)
    : "Refreshing enabled packages";
  const overlayDetail = liveOperation
    ? formatRefreshDetail(liveOperation)
    : "Preparing package refresh…";
  const overlayProgress = liveOperation
    ? liveOperation.total_packages === 0
      ? liveOperation.state === "running"
        ? 0
        : 100
      : Math.min(
          100,
          Math.round(
            (liveOperation.processed_packages / liveOperation.total_packages) * 100,
          ),
        )
    : 0;

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
      <FilterBar
        activeCount={
          (search ? 1 : 0) + (enabledFilter !== "all" ? 1 : 0)
        }
        onClear={() => {
          setSearchInput("");
          setSearch("");
          setEnabledFilter("all");
        }}
        trailing={
          <Button
            variant="secondary"
            size="md"
            className="w-full xl:w-auto"
            onClick={applyFilters}
          >
            Apply Filters
          </Button>
        }
      >
        <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_220px]">
          <label className="block">
            <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-400">
              Search
            </span>
            <input
              type="text"
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && applyFilters()}
              placeholder="Filter by name or description"
              className="w-full border-2 border-zinc-700 bg-black px-4 py-2.5 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-[var(--theme-accent-lime)] focus:ring-2 focus:ring-[var(--theme-accent-lime)]"
            />
          </label>
          <label className="block">
            <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-zinc-400">
              Status
            </span>
            <Select
              value={enabledFilter}
              onValueChange={(val) => setEnabledFilter(val as EnabledFilter)}
              options={[
                { value: "all", label: "All" },
                { value: "true", label: "Enabled" },
                { value: "false", label: "Disabled" },
              ]}
            />
          </label>
        </div>
      </FilterBar>

      {/* Package Cards */}
      {listQuery.data.packages.length === 0 ? (
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
          {listQuery.data.packages.map((entry) => (
            <PackageCard
              key={entry.package.name}
              entry={entry}
              onRefresh={(name) => trigger(name, "refresh")}
              onRebuild={(name) => trigger(name, "rebuild")}
              onDelete={(name) => void handleDelete(name)}
              refreshing={refreshingNameForMutation === entry.package.name}
              refreshDisabled={refreshingAll}
            />
          ))}
        </div>
      )}

      {/* Pagination */}
      {listQuery.data.packages.length > 0 && (
        <div className="border-2 border-white bg-black p-4">
          <PaginationControls
            onPrevious={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
            onNext={() => setOffset(offset + PAGE_SIZE)}
            previousDisabled={listQuery.isFetching || offset === 0}
            nextDisabled={
              listQuery.isFetching || !listQuery.data.page.has_more
            }
            summary={`Offset: ${offset}`}
          />
        </div>
      )}

      {showAddModal && (
        <AddPackageModal
          onClose={() => setShowAddModal(false)}
          onSuccess={() => {
            setShowAddModal(false);
            void invalidatePackages();
          }}
        />
      )}

      <ProgressOverlayDialog
        open={refreshOverlayOpen}
        title={overlayTitle}
        detail={overlayDetail}
        progress={overlayProgress}
        onClose={() => setRefreshOverlayOpen(false)}
        closeDisabled={refreshingAll}
      />
    </div>
  );
}

export default function PackageListPage() {
  return <PackageList />;
}
