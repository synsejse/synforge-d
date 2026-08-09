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
import { useBulkAction } from "./use-bulk-action";
import PackageCard from "./components/package-card";
import ErrorMessage from "../../components/common/error-message";
import { useDialogs } from "../../components/common/dialogs-context";
import { useToast } from "../../components/common/toast-context";
import LoadingBlock from "../../components/ui/loading-block";
import FaIcon from "../../components/ui/fa-icon";
import Button from "../../components/ui/button";
import Select from "../../components/ui/select";
import PageHeader from "../../components/ui/page-header";
import ProgressOverlayDialog from "../../components/ui/progress-overlay-dialog";
import PaginationControls from "../../components/common/pagination-controls";
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
    <div className="grid grid-cols-2 gap-px border border-edge bg-edge-strong">
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

  const bulkAction = useBulkAction(invalidatePackages);

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

  async function handleBulkRefresh() {
    const names = Array.from(selection.selected);
    if (names.length === 0) return;
    const ok = await confirm({
      title: `Refresh ${names.length} package${names.length === 1 ? "" : "s"}?`,
      message: "Queue a manual source refresh for each selected package.",
      confirmLabel: "Refresh",
    });
    if (!ok) return;
    await bulkAction.run({
      items: names,
      action: (name) => api.refreshPackage(name),
      successTitle: "Bulk refresh queued",
      partialTitle: "Bulk refresh partial",
      successMessage: (count) =>
        `${count} package${count === 1 ? "" : "s"} processed.`,
    });
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
    await bulkAction.run({
      items: names,
      action: (name) => api.rebuildPackage(name),
      successTitle: "Bulk rebuild queued",
      partialTitle: "Bulk rebuild partial",
      successMessage: (count) =>
        `${count} package${count === 1 ? "" : "s"} processed.`,
    });
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
    await bulkAction.run({
      items: names,
      action: (name) => api.deletePackage(name),
      successTitle: "Packages deleted",
      partialTitle: "Bulk delete partial",
      successMessage: (count) =>
        `${count} package${count === 1 ? "" : "s"} removed.`,
    });
    selection.clear();
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
    <div className="space-y-6">
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

      <div className="grid items-end gap-4 border border-edge bg-black p-[18px] sm:grid-cols-[minmax(0,1fr)_200px]">
        <label className="block">
          <span className="block font-mono text-[9px] font-semibold uppercase tracking-[0.22em] text-soft">
            Search
          </span>
          <input
            type="text"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            placeholder="Filter by name or description"
            className="mt-2.5 w-full border border-edge bg-black px-3 py-2.5 font-mono text-[13px] text-white outline-none transition-colors focus:border-accent-lime"
          />
        </label>
        <label className="block">
          <span className="block font-mono text-[9px] font-semibold uppercase tracking-[0.22em] text-soft">
            Status
          </span>
          <div className="mt-2.5">
            <Select
              value={enabledFilter}
              onValueChange={(val) => setEnabledFilter(val as EnabledFilter)}
              options={[
                { value: "all", label: "All" },
                { value: "true", label: "Enabled" },
                { value: "false", label: "Disabled" },
              ]}
            />
          </div>
        </label>
      </div>

      {!loading && packages.length > 0 ? (
        <div className="flex items-center justify-between gap-3 border border-edge bg-[#09090b] px-4 py-3 font-mono text-[10px] uppercase tracking-[0.14em] text-soft">
          <label className="flex items-center gap-2.5 hover:text-white">
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
        <div className="border border-edge bg-black p-12 text-center">
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
        <PaginationControls
          offset={offset}
          pageSize={PAGE_SIZE}
          count={packages.length}
          hasMore={listQuery.data.page.has_more}
          total={listQuery.data.page.total}
          isFetching={listQuery.isFetching}
          onOffsetChange={setOffset}
        />
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
          disabled={bulkAction.running}
        >
          <FaIcon icon={faRotate} />
          Refresh
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleBulkRebuild}
          disabled={bulkAction.running}
        >
          <FaIcon icon={faHammer} />
          Rebuild
        </Button>
        <Button
          variant="danger"
          size="sm"
          onClick={handleBulkDelete}
          disabled={bulkAction.running}
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
