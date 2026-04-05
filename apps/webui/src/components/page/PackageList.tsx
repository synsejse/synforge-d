import { useEffect, useState } from "react";
import { faPlus } from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { summarizePackageAction } from "../../lib/package-actions";
import type { PackageResponse } from "../../lib/types";
import AddPackageModal from "../package/AddPackageModal";
import ErrorMessage from "../common/ErrorMessage";
import PaginationControls from "../common/PaginationControls";
import EmptyState from "../ui/EmptyState";
import LoadingBlock from "../ui/LoadingBlock";
import PackageCard from "../package-list/PackageCard";
import PackageFilters from "../package-list/PackageFilters";
import PageHeader from "../ui/PageHeader";

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
  const [enabledFilter, setEnabledFilter] = useState<"all" | "true" | "false">(
    () => {
      if (typeof window === "undefined") {
        return "all";
      }
      const value = new URLSearchParams(window.location.search).get("enabled");
      return value === "true" || value === "false" ? value : "all";
    },
  );
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);
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
    if (!confirm(`Delete package "${name}"?`)) {
      return;
    }
    try {
      await api.deletePackage(name);
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : "Failed to delete package");
    }
  }

  async function trigger(name: string, action: "refresh" | "rebuild") {
    try {
      const response =
        action === "refresh"
          ? await api.refreshPackage(name)
          : await api.rebuildPackage(name);
      alert(summarizePackageAction(response));
      await load();
    } catch (e) {
      alert(e instanceof Error ? e.message : `Failed to ${action} package`);
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
      <PageHeader
        eyebrow="Package Registry"
        title="Packages"
        description="Sources, targets, and builds."
        actions={[
          {
            onClick: () => setShowAddModal(true),
            label: "Add Package",
            icon: faPlus,
            variant: "primary",
          },
        ]}
      />

      <PackageFilters
        search={search}
        enabledFilter={enabledFilter}
        onSearchChange={setSearch}
        onEnabledFilterChange={setEnabledFilter}
        onApply={() => load(0, search, enabledFilter)}
      />

      {packages.length === 0 ? (
        <EmptyState>
          No packages configured yet. Add a spec source to start building.
        </EmptyState>
      ) : (
        <div className="space-y-4">
          {packages.map((entry) => (
            <PackageCard
              key={entry.package.name}
              entry={entry}
              onRefresh={(name) => void trigger(name, "refresh")}
              onRebuild={(name) => void trigger(name, "rebuild")}
              onDelete={(name) => void handleDelete(name)}
            />
          ))}
        </div>
      )}

      {packages.length > 0 ? (
        <PaginationControls
          onPrevious={() => load(Math.max(0, offset - pageSize), search, enabledFilter)}
          onNext={() => load(offset + pageSize, search, enabledFilter)}
          previousDisabled={loading || offset === 0}
          nextDisabled={loading || !hasMore}
        />
      ) : null}

      {showAddModal && (
        <AddPackageModal
          onClose={() => setShowAddModal(false)}
          onSuccess={() => {
            setShowAddModal(false);
            load();
          }}
        />
      )}
    </div>
  );
}
