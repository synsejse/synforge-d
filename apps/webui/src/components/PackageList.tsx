import { useEffect, useState } from "react";
import api from "../lib/api";
import { summarizePackageAction } from "../lib/package-actions";
import ActionButton from "./ActionButton";
import AddPackageModal from "./AddPackageModal";
import EmptyState from "./EmptyState";
import FaIcon from "./FaIcon";
import LoadingBlock from "./LoadingBlock";
import PageHeader from "./PageHeader";
import StatusPill from "./StatusPill";
import type { PackageResponse, PackageTargetState } from "../lib/types";
import {
  faFolderOpen,
  faPlus,
  faRotate,
  faHammer,
  faTrash,
  faMagnifyingGlass,
} from "@fortawesome/free-solid-svg-icons";

function summarizePackageStatus(entry: PackageResponse) {
  if (!entry.package.enabled) {
    return "disabled";
  }
  if (entry.state.targets.some((target) => target.active_status === "running")) {
    return "running";
  }
  if (entry.state.targets.some((target) => target.active_status === "pending")) {
    return "pending";
  }
  return "enabled";
}

function targetStatus(target: PackageTargetState) {
  if (target.active_status) {
    return target.active_status;
  }
  return target.last_successful_build_id ? "succeeded" : "disabled";
}

function compactRevision(revision: string | null) {
  if (!revision) {
    return "No successful revision";
  }
  if (revision.length <= 44) {
    return revision;
  }
  return `${revision.slice(0, 20)}...${revision.slice(-16)}`;
}

export default function PackageList() {
  const [packages, setPackages] = useState<PackageResponse[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [offset, setOffset] = useState(() => {
    if (typeof window === "undefined") {
      return 0;
    }
    return Number(
      new URLSearchParams(window.location.search).get("offset") || "0",
    );
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
        window.history.replaceState(
          {},
          "",
          `/packages/${query ? `?${query}` : ""}`,
        );
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
    return (
      <div className="border border-zinc-800 bg-black p-4 text-zinc-200">
        Error: {error}
      </div>
    );
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

      <section className="grid gap-3 border border-zinc-800 bg-black p-4 md:grid-cols-[minmax(0,1fr)_220px_auto]">
        <label className="block">
          <span className="sr-only">Search packages</span>
          <input
            type="search"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Search name or description"
            className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
          />
        </label>
        <label className="block">
          <span className="sr-only">Filter by enabled state</span>
          <select
            value={enabledFilter}
            onChange={(event) =>
              setEnabledFilter(event.target.value as "all" | "true" | "false")
            }
            className="w-full border border-zinc-800 bg-black px-4 py-2.5 text-sm text-white outline-none transition focus:border-zinc-600"
          >
            <option value="all">All states</option>
            <option value="true">Enabled only</option>
            <option value="false">Disabled only</option>
          </select>
        </label>
        <button
          type="button"
          onClick={() => load(0, search, enabledFilter)}
          className="border border-zinc-800 bg-black px-4 py-2.5 text-sm text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
        >
          <FaIcon icon={faMagnifyingGlass} className="mr-2" />
          Apply
        </button>
      </section>

      {packages.length === 0 ? (
        <EmptyState>
          No packages configured yet. Add a spec source to start building.
        </EmptyState>
      ) : (
        <div className="space-y-4">
          {packages.map((entry) => {
            const status = summarizePackageStatus(entry);
            return (
              <article
                key={entry.package.name}
                className="border border-zinc-800 bg-black"
              >
                <div className="flex flex-col gap-5 border-b border-zinc-800 px-5 py-5 xl:flex-row xl:items-start xl:justify-between">
                  <div className="min-w-0 flex-1 space-y-4">
                    <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                      <div className="min-w-0">
                        <a
                          href={`/packages/view/?name=${encodeURIComponent(entry.package.name)}`}
                          className="text-lg font-semibold text-white transition hover:text-zinc-300"
                        >
                          {entry.package.name}
                        </a>
                        <div className="mt-1 max-w-3xl text-sm text-zinc-500">
                          {entry.package.description || "No description"}
                        </div>
                      </div>
                      <div className="flex items-center gap-3">
                        <StatusPill status={status} />
                      </div>
                    </div>

                    <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                      <div className="border border-zinc-800 bg-zinc-950/40 px-4 py-3">
                        <div className="text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                          Version
                        </div>
                        <div className="mt-2 text-sm text-zinc-200">
                          {entry.package.version}-{entry.package.release}
                        </div>
                      </div>
                      <div className="border border-zinc-800 bg-zinc-950/40 px-4 py-3">
                        <div className="text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                          Targets
                        </div>
                        <div className="mt-2 text-sm text-zinc-300">
                          {formatMockChroots(entry.package.mock_chroots)}
                        </div>
                      </div>
                      <div className="border border-zinc-800 bg-zinc-950/40 px-4 py-3 md:col-span-2">
                        <div className="text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                          Repository
                        </div>
                        <div className="mt-2 break-all font-mono text-sm text-zinc-300">
                          {entry.package.source.repo_url}
                        </div>
                      </div>
                      <div className="border border-zinc-800 bg-zinc-950/40 px-4 py-3 md:col-span-2">
                        <div className="text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                          Spec File
                        </div>
                        <div className="mt-2 break-all font-mono text-sm text-zinc-300">
                          {entry.package.source.spec_file}
                        </div>
                      </div>
                      <div className="border border-zinc-800 bg-zinc-950/40 px-4 py-3 md:col-span-2">
                        <div className="text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                          Last Revision
                        </div>
                        <div className="mt-2 break-all font-mono text-sm text-zinc-400">
                          {entry.state.last_revision || "None yet"}
                        </div>
                      </div>
                    </div>
                  </div>

                  <div className="flex shrink-0 flex-wrap justify-end gap-2 xl:max-w-[460px]">
                    <ActionButton
                      href={`/packages/view/?name=${encodeURIComponent(entry.package.name)}`}
                      icon={faFolderOpen}
                      aria-label={`Open package ${entry.package.name}`}
                    >
                      Open
                    </ActionButton>
                    <ActionButton
                      onClick={() => trigger(entry.package.name, "refresh")}
                      icon={faRotate}
                      aria-label={`Refresh package ${entry.package.name}`}
                    >
                      Refresh
                    </ActionButton>
                    <ActionButton
                      onClick={() => trigger(entry.package.name, "rebuild")}
                      icon={faHammer}
                      aria-label={`Rebuild package ${entry.package.name}`}
                    >
                      Rebuild
                    </ActionButton>
                    <ActionButton
                      onClick={() => handleDelete(entry.package.name)}
                      icon={faTrash}
                      aria-label={`Delete package ${entry.package.name}`}
                      className="text-zinc-300"
                    >
                      Delete
                    </ActionButton>
                  </div>
                </div>

                <div className="px-5 py-4">
                  <div className="mb-3 text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                    Target State
                  </div>
                  <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                    {entry.state.targets.map((target) => (
                      <div
                        key={`${entry.package.name}:${target.mock_chroot}`}
                        className="border border-zinc-800 bg-zinc-950/40 px-4 py-3"
                      >
                        <div className="flex items-start justify-between gap-3">
                          <div className="min-w-0">
                            <div className="font-mono text-sm text-zinc-100">
                              {target.mock_chroot}
                            </div>
                            <div className="mt-1 text-xs text-zinc-500">
                              {target.active_job_id
                                ? `Active job ${target.active_job_id}`
                                : target.last_successful_build_id
                                  ? `Last success ${target.last_successful_build_id}`
                                  : "No successful build yet"}
                            </div>
                          </div>
                          <StatusPill status={targetStatus(target)} />
                        </div>
                        <div className="mt-3 border-t border-zinc-800 pt-3">
                          <div className="text-[11px] uppercase tracking-[0.18em] text-zinc-500">
                            Revision
                          </div>
                          <div className="mt-2 break-all font-mono text-sm text-zinc-400">
                            {compactRevision(target.last_revision)}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </article>
            );
          })}
        </div>
      )}

      {packages.length > 0 ? (
        <div className="flex items-center justify-between gap-3">
          <button
            onClick={() =>
              load(Math.max(0, offset - pageSize), search, enabledFilter)
            }
            disabled={loading || offset === 0}
            className="border border-zinc-800 px-4 py-2 text-sm text-zinc-300 disabled:cursor-not-allowed disabled:opacity-50"
          >
            Previous
          </button>
          <button
            onClick={() => load(offset + pageSize, search, enabledFilter)}
            disabled={loading || !hasMore}
            className="border border-zinc-800 px-4 py-2 text-sm text-zinc-300 disabled:cursor-not-allowed disabled:opacity-50"
          >
            Next
          </button>
        </div>
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

function formatMockChroots(chroots: string[]) {
  return chroots.join(", ");
}
