import { useEffect, useState } from "react";
import { useDebounce } from "../../lib/hooks/use-debounce";
import { useSelection } from "../../lib/hooks/use-selection";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
import {
  faHammer,
  faPlus,
  faRotate,
  faTrash,
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
import FaIcon from "../../components/ui/fa-icon";
import Button from "../../components/ui/button";
import Select from "../../components/ui/select";
import PageHeader from "../../components/ui/page-header";
import ProgressOverlayDialog from "../../components/ui/progress-overlay-dialog";
import PaginationControls from "../../components/common/pagination-controls";
import FilterBar from "../../components/common/filter-bar";
import SelectionActionBar from "../../components/common/selection-action-bar";

const PAGE_SIZE = 50;

type EnabledFilter = "all" | "true" | "false";

function refreshTitle(state: RefreshAllPackagesProgressView["state"]): string {
  if (state === "running") return "Refreshing enabled packages";
  if (state === "failed") return "Refresh all failed";
  return "Refresh all complete";
}

function refreshTone(
  state: RefreshAllPackagesProgressView["state"],
): "running" | "success" | "error" {
  if (state === "failed") return "error";
  if (state === "running") return "running";
  return "success";
}

function StatRow({
  label,
  value,
  emphasis,
}: {
  label: string;
  value: number;
  emphasis?: "lime" | "error" | "muted";
}) {
  const valueClass =
    value === 0
      ? "text-soft"
      : emphasis === "error"
        ? "text-error"
        : emphasis === "lime"
          ? "text-accent-lime"
          : "text-white";
  return (
    <div className="flex items-baseline justify-between gap-3">
      <span className="font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-soft">
        {label}
      </span>
      <span className={`font-mono text-sm font-bold ${valueClass}`}>{value}</span>
    </div>
  );
}

function RefreshAllStats({
  operation,
}: {
  operation: RefreshAllPackagesProgressView;
}) {
  return (
    <div className="grid grid-cols-2 gap-px border-2 border-edge-strong bg-edge-strong">
      <div className="space-y-2 bg-black px-4 py-3">
        <div className="font-mono text-[10px] font-bold uppercase tracking-[0.24em] text-soft">
          Packages
        </div>
        <StatRow label="Queued" value={operation.queued_packages} emphasis="lime" />
        <StatRow label="Skipped" value={operation.skipped_packages} />
        <StatRow label="Blocked" value={operation.blocked_packages} />
        <StatRow
          label="Failed"
          value={operation.failed_packages}
          emphasis="error"
        />
      </div>
      <div className="space-y-2 bg-black px-4 py-3">
        <div className="font-mono text-[10px] font-bold uppercase tracking-[0.24em] text-soft">
          Targets
        </div>
        <StatRow label="Queued" value={operation.queued_targets} emphasis="lime" />
        <StatRow label="Skipped" value={operation.skipped_targets} />
        <StatRow label="Blocked" value={operation.blocked_targets} />
      </div>
    </div>
  );
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
  const debouncedSearch = useDebounce(searchInput, 250);
  const [showAddModal, setShowAddModal] = useState(false);
  const [refreshOverlayOpen, setRefreshOverlayOpen] = useState(false);
  const [bulkRunning, setBulkRunning] = useState(false);
  const selection = useSelection<string>();

  const setOffset = (next: number) =>
    navigate({ search: (prev) => ({ ...prev, offset: next }) });
  const setSearch = (next: string) => {
    selection.clear();
    navigate({ search: (prev) => ({ ...prev, offset: 0, search: next }) });
  };
  const setEnabledFilter = (next: EnabledFilter) => {
    selection.clear();
    navigate({ search: (prev) => ({ ...prev, offset: 0, enabled: next }) });
  };

  useEffect(() => {
    if (debouncedSearch !== search) {
      setSearch(debouncedSearch);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedSearch]);

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

  async function runBulk(
    action: "refresh" | "rebuild",
    names: string[],
  ): Promise<void> {
    setBulkRunning(true);
    try {
      const results = await Promise.allSettled(
        names.map((name) =>
          action === "refresh"
            ? api.refreshPackage(name)
            : api.rebuildPackage(name),
        ),
      );
      const ok = results.filter((r) => r.status === "fulfilled").length;
      const failed = results.length - ok;
      if (failed === 0) {
        toast.success(
          action === "refresh" ? "Bulk refresh queued" : "Bulk rebuild queued",
          `${ok} package${ok === 1 ? "" : "s"} processed.`,
        );
      } else {
        toast.error(
          action === "refresh"
            ? "Bulk refresh partial"
            : "Bulk rebuild partial",
          `${ok} succeeded · ${failed} failed.`,
        );
      }
      await invalidatePackages();
    } finally {
      setBulkRunning(false);
    }
  }

  async function handleBulkRefresh() {
    const names = Array.from(selection.selected);
    if (names.length === 0) return;
    const ok = await confirm({
      title: `Refresh ${names.length} package${names.length === 1 ? "" : "s"}?`,
      message: "Queue a manual source refresh for each selected package.",
      confirmLabel: "Refresh",
    });
    if (!ok) return;
    await runBulk("refresh", names);
    selection.clear();
  }

  async function handleBulkRebuild() {
    const names = Array.from(selection.selected);
    if (names.length === 0) return;
    const ok = await confirm({
      title: `Rebuild ${names.length} package${names.length === 1 ? "" : "s"}?`,
      message: "Queue a fresh build of every target for each selected package.",
      confirmLabel: "Rebuild",
    });
    if (!ok) return;
    await runBulk("rebuild", names);
    selection.clear();
  }

  async function handleBulkDelete() {
    const names = Array.from(selection.selected);
    if (names.length === 0) return;
    const ok = await confirm({
      title: `Delete ${names.length} package${names.length === 1 ? "" : "s"}?`,
      message: "All selected packages will be removed. This cannot be undone.",
      confirmLabel: "Delete",
      destructive: true,
    });
    if (!ok) return;
    setBulkRunning(true);
    try {
      const results = await Promise.allSettled(
        names.map((name) => api.deletePackage(name)),
      );
      const ok2 = results.filter((r) => r.status === "fulfilled").length;
      const failed = results.length - ok2;
      if (failed === 0) {
        toast.success(
          "Packages deleted",
          `${ok2} package${ok2 === 1 ? "" : "s"} removed.`,
        );
      } else {
        toast.error(
          "Bulk delete partial",
          `${ok2} succeeded · ${failed} failed.`,
        );
      }
      await invalidatePackages();
    } finally {
      setBulkRunning(false);
      selection.clear();
    }
  }

  function applyFilters() {
    setSearch(searchInput);
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

  const loading = listQuery.isPending;
  const packages = listQuery.data?.packages ?? [];

  const refreshingNameForMutation =
    triggerMutation.isPending &&
    triggerMutation.variables?.action === "refresh"
      ? triggerMutation.variables.name
      : null;

  const overlayTitle = liveOperation
    ? refreshTitle(liveOperation.state)
    : "Refreshing enabled packages";
  const overlayTone = liveOperation ? refreshTone(liveOperation.state) : "running";
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
  const overlaySummary = liveOperation
    ? liveOperation.total_packages === 0
      ? "Preparing…"
      : `${liveOperation.processed_packages} / ${liveOperation.total_packages} packages`
    : "Preparing…";
  // Only surface the message line when it's not redundant with the
  // stats (e.g. an error string while failed, or "collecting enabled
  // packages" pre-roll). Plain progress noise gets suppressed.
  const overlayMessage =
    liveOperation?.message && liveOperation.state !== "completed"
      ? liveOperation.message
      : null;

  return (
    <div className="space-y-8">
      <PageHeader
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
            <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-muted">
              Search
            </span>
            <input
              type="text"
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && applyFilters()}
              placeholder="Filter by name or description"
              className="w-full border-2 border-edge-strong bg-black px-4 py-2.5 font-mono text-sm text-white outline-none transition duration-100 ease-linear focus:border-accent-lime focus:ring-2 focus:ring-accent-lime"
            />
          </label>
          <label className="block">
            <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.2em] text-muted">
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

      {!loading && packages.length > 0 ? (
        <div className="flex items-center justify-between gap-3 border-2 border-edge-strong bg-surface-alt px-4 py-2 font-mono text-xs uppercase tracking-[0.15em]">
          <label className="flex items-center gap-3 text-muted hover:text-white">
            <input
              type="checkbox"
              checked={selection.allSelected(
                packages.map((p) => p.package.name),
              )}
              ref={(el) => {
                if (el) {
                  el.indeterminate = selection.someSelected(
                    packages.map((p) => p.package.name),
                  );
                }
              }}
              onChange={(event) =>
                selection.setMany(
                  packages.map((p) => p.package.name),
                  event.target.checked,
                )
              }
              aria-label="Select all packages on this page"
            />
            Select all on page ({packages.length})
          </label>
          {selection.count > 0 ? (
            <span className="text-soft">
              {selection.count} total selected
            </span>
          ) : null}
        </div>
      ) : null}

      {loading ? (
        <LoadingBlock label="Loading packages…" lines={4} />
      ) : packages.length === 0 ? (
        <div className="border-2 border-edge-strong bg-black p-12 text-center">
          <p className="font-mono text-sm font-bold uppercase tracking-[0.3em] text-soft">
            NO_PACKAGES_CONFIGURED
          </p>
          <p className="mt-2 text-sm text-soft">
            Add a spec source to start building.
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {packages.map((entry) => (
            <PackageCard
              key={entry.package.name}
              entry={entry}
              onRefresh={(name) => trigger(name, "refresh")}
              onRebuild={(name) => trigger(name, "rebuild")}
              onDelete={(name) => void handleDelete(name)}
              refreshing={refreshingNameForMutation === entry.package.name}
              refreshDisabled={refreshingAll}
              selected={selection.isSelected(entry.package.name)}
              onToggleSelected={selection.setOne}
            />
          ))}
        </div>
      )}

      {!loading && listQuery.data && packages.length > 0 && (
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
        tone={overlayTone}
        summary={overlaySummary}
        progress={overlayProgress}
        onClose={() => setRefreshOverlayOpen(false)}
        closeDisabled={refreshingAll}
      >
        {liveOperation && liveOperation.total_packages > 0 ? (
          <RefreshAllStats operation={liveOperation} />
        ) : null}
        {overlayMessage ? (
          <p
            className={`mt-4 font-mono text-xs ${
              overlayTone === "error" ? "text-error" : "text-soft"
            }`}
          >
            {overlayMessage}
          </p>
        ) : null}
      </ProgressOverlayDialog>

      <SelectionActionBar
        count={selection.count}
        noun={{ singular: "package", plural: "packages" }}
        onClear={selection.clear}
      >
        <Button
          variant="ghost"
          size="sm"
          onClick={handleBulkRefresh}
          disabled={bulkRunning}
        >
          <FaIcon icon={faRotate} />
          Refresh
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleBulkRebuild}
          disabled={bulkRunning}
        >
          <FaIcon icon={faHammer} />
          Rebuild
        </Button>
        <Button
          variant="danger"
          size="sm"
          onClick={handleBulkDelete}
          disabled={bulkRunning}
        >
          <FaIcon icon={faTrash} />
          Delete
        </Button>
      </SelectionActionBar>
    </div>
  );
}

export default function PackageListPage() {
  return <PackageList />;
}
