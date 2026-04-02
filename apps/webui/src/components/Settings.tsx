import { useEffect, useState, type FormEvent } from "react";
import api from "../lib/api";
import type { DaemonConfig } from "../lib/types";
import PageHeader from "./PageHeader";
import { faSave, faServer } from "@fortawesome/free-solid-svg-icons";
import FaIcon from "./FaIcon";

export default function Settings() {
  const [config, setConfig] = useState<DaemonConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [publicBaseUrl, setPublicBaseUrl] = useState("");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    async function load() {
      try {
        const res = await api.getConfig();
        setConfig(res.config);
        setPublicBaseUrl(res.config.public_base_url);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load config");
      } finally {
        setLoading(false);
      }
    }

    load();
  }, []);

  async function handleSave(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    try {
      const res = await api.updateRuntimeSettings({ public_base_url: publicBaseUrl.trim() });
      setConfig(res.config);
      setPublicBaseUrl(res.config.public_base_url);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to update settings");
    } finally {
      setSaving(false);
    }
  }

  if (loading) {
    return <div className="text-zinc-400">Loading config…</div>;
  }

  if (error || !config) {
    return <div className="border border-zinc-800 bg-black p-4 text-zinc-200">Error: {error || "Failed to load"}</div>;
  }

  const sections = [
    {
      title: "Env-backed Server",
      items: [
        { label: "Listen address", value: config.listen_addr },
        { label: "Bearer token", value: config.bearer_token ? "Configured" : "(none)" },
      ],
    },
    {
      title: "Storage",
      items: [
        { label: "Runtime root", value: config.runtime_root },
        { label: "Database path", value: config.database_path },
        { label: "Package metadata", value: config.packages_dir },
        { label: "Fedora repo root", value: config.repo_dir },
        { label: "Jobs root", value: config.jobs_root },
      ],
    },
    {
      title: "Worker",
      items: [
        { label: "Worker image", value: config.worker_image },
        { label: "Worker listen", value: config.worker_listen_addr },
        { label: "Worker connect", value: config.worker_connect_addr },
      ],
    },
    {
      title: "Build settings",
      items: [
        { label: "Max concurrent builds", value: String(config.max_concurrent_builds) },
        { label: "Package polling", value: "Configured per package" },
        { label: "Build timeout", value: "Configured per package" },
      ],
    },
  ];

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Daemon Settings"
        title="Configuration"
        description="Runtime settings and effective daemon values."
        actions={[{ href: "/", label: "Overview", icon: faServer }]}
      />

      <form onSubmit={handleSave} className="border border-zinc-800 bg-black p-6">
        <div className="mb-6">
          <h2 className="text-xl font-semibold text-white">Runtime Settings</h2>
          <p className="mt-2 text-sm text-zinc-400">Settings editable from the UI.</p>
        </div>

        <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_auto]">
          <label className="block">
            <span className="mb-2 block text-sm font-medium text-zinc-300">Public base URL</span>
            <input
              type="url"
              value={publicBaseUrl}
              onChange={(event) => setPublicBaseUrl(event.target.value)}
              className="w-full border border-zinc-800 bg-black px-4 py-3 text-sm text-white outline-none transition focus:border-zinc-600"
              required
            />
          </label>
          <div className="flex items-end">
            <button
              type="submit"
              disabled={saving}
              className="border border-zinc-200 bg-zinc-100 px-5 py-3 text-sm font-semibold text-black transition hover:bg-white disabled:opacity-70"
            >
              <FaIcon icon={faSave} className="mr-2" />
              {saving ? "Saving…" : "Save"}
            </button>
          </div>
        </div>
      </form>

      <div className="grid gap-5 xl:grid-cols-2">
        {sections.map((section) => (
          <section
            key={section.title}
            className="border border-zinc-800 bg-black p-6"
          >
            <h2 className="text-xl font-semibold text-white">{section.title}</h2>
            <dl className="mt-5 space-y-3">
              {section.items.map((item) => (
                <div
                  key={item.label}
                  className="flex flex-col gap-2 border border-zinc-800 bg-black px-4 py-3"
                >
                  <dt className="text-xs uppercase tracking-[0.18em] text-zinc-500">{item.label}</dt>
                  <dd className="break-all font-mono text-sm text-zinc-200">{item.value}</dd>
                </div>
              ))}
            </dl>
          </section>
        ))}
      </div>
    </div>
  );
}
