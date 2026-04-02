import { useEffect, useMemo, useState } from "react";
import api from "../lib/api";
import PageHeader from "./PageHeader";
import FaIcon from "./FaIcon";
import {
  faCopy,
  faDownload,
  faFolderTree,
  faTerminal,
} from "@fortawesome/free-solid-svg-icons";

type ShellKind = "bash" | "zsh" | "fish";

export default function RepositorySetup() {
  const [publicBaseUrl, setPublicBaseUrl] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [copiedLabel, setCopiedLabel] = useState<string | null>(null);
  const [selectedShell, setSelectedShell] = useState<ShellKind>("bash");

  useEffect(() => {
    async function load() {
      try {
        setLoading(true);
        const configRes = await api.getConfig();
        setPublicBaseUrl(normalizeBaseUrl(configRes.config.public_base_url));
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to load repository setup");
      } finally {
        setLoading(false);
      }
    }

    load();
  }, []);

  const repoBaseUrl = useMemo(() => {
    if (!publicBaseUrl) {
      return "";
    }
    return `${publicBaseUrl}/repo`;
  }, [publicBaseUrl]);

  const repoFileContents = useMemo(() => buildRepoFile(repoBaseUrl), [repoBaseUrl]);
  const writeRepoCommands = useMemo(
    () => buildRepoWriteCommands(repoFileContents),
    [repoFileContents]
  );
  const installCommand = "sudo dnf install <package-name>";

  async function copy(label: string, value: string) {
    try {
      await navigator.clipboard.writeText(value);
      setCopiedLabel(label);
      window.setTimeout(() => {
        setCopiedLabel((current) => (current === label ? null : current));
      }, 1500);
    } catch {
      setCopiedLabel(null);
    }
  }

  function downloadRepoFile() {
    const blob = new Blob([repoFileContents], { type: "text/plain;charset=utf-8" });
    const url = window.URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = "synforge.repo";
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.URL.revokeObjectURL(url);
  }

  if (loading) {
    return <div className="text-zinc-400">Loading repository setup…</div>;
  }

  if (error) {
    return <div className="border border-zinc-800 bg-black p-4 text-zinc-200">Error: {error}</div>;
  }

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="Repository Access"
        title="Add Repo To Fedora"
        description="Repo file and basic DNF usage."
        actions={[{ href: "/repository/", label: "Browse Repository", icon: faFolderTree }]}
      />

      <section className="border border-zinc-800 bg-black p-6">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 className="text-xl font-semibold text-white">Repo File</h2>
            <p className="mt-2 text-sm text-zinc-400">
              Save <span className="font-mono text-zinc-200">synforge.repo</span> into
              <span className="font-mono text-zinc-200"> /etc/yum.repos.d/</span>.
            </p>
          </div>
          <button
            type="button"
            onClick={downloadRepoFile}
            className="border border-zinc-200 bg-zinc-100 px-4 py-2 text-sm font-semibold text-black transition hover:bg-white"
          >
            <FaIcon icon={faDownload} className="mr-2" />
            Download repo file
          </button>
        </div>

        <div className="mt-5 border border-zinc-800 bg-zinc-950">
          <CodeBlock
            label="synforge.repo"
            value={repoFileContents}
            copied={copiedLabel === "repo-file"}
            onCopy={() => copy("repo-file", repoFileContents)}
          />
        </div>
      </section>

      <section className="grid gap-4 xl:grid-cols-2">
        <article className="border border-zinc-800 bg-black p-6">
          <div className="flex items-center gap-2 text-sm font-medium text-white">
            <FaIcon icon={faTerminal} />
            Install repo file
          </div>
          <div className="mt-4 flex flex-wrap gap-2">
            {(["bash", "zsh", "fish"] as ShellKind[]).map((shell) => (
              <button
                key={shell}
                type="button"
                onClick={() => setSelectedShell(shell)}
                className={[
                  "border px-3 py-1.5 text-xs uppercase tracking-[0.18em] transition",
                  selectedShell === shell
                    ? "border-zinc-200 bg-zinc-100 text-black"
                    : "border-zinc-800 bg-black text-zinc-300 hover:border-zinc-600 hover:bg-zinc-950",
                ].join(" ")}
              >
                {shell}
              </button>
            ))}
          </div>
          <div className="mt-4 border border-zinc-800 bg-zinc-950">
            <CodeBlock
              label={`${selectedShell} · write synforge.repo`}
              value={writeRepoCommands[selectedShell]}
              copied={copiedLabel === `write-command-${selectedShell}`}
              onCopy={() => copy(`write-command-${selectedShell}`, writeRepoCommands[selectedShell])}
            />
          </div>
        </article>

        <article className="border border-zinc-800 bg-black p-6">
          <div className="flex items-center gap-2 text-sm font-medium text-white">
            <FaIcon icon={faTerminal} />
            Use repo
          </div>
          <div className="mt-4 border border-zinc-800 bg-zinc-950">
            <CodeBlock
              label="usage"
              value={`sudo dnf clean all\nsudo dnf makecache\n${installCommand}`}
              copied={copiedLabel === "usage-command"}
              onCopy={() => copy("usage-command", `sudo dnf clean all\nsudo dnf makecache\n${installCommand}`)}
            />
          </div>
        </article>
      </section>
    </div>
  );
}

function CodeBlock({
  label,
  value,
  copied,
  onCopy,
}: {
  label: string;
  value: string;
  copied: boolean;
  onCopy: () => void;
}) {
  return (
    <div>
      <div className="flex items-center justify-between border-b border-zinc-800 px-4 py-3">
        <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">{label}</div>
        <button
          type="button"
          onClick={onCopy}
          className="border border-zinc-800 bg-black px-3 py-1.5 text-xs font-medium text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-950"
        >
          <FaIcon icon={faCopy} className="mr-2" />
          {copied ? "Copied" : "Copy"}
        </button>
      </div>
      <pre className="overflow-x-auto px-4 py-4 text-sm leading-6 text-zinc-200">
        <code>{value}</code>
      </pre>
    </div>
  );
}

function buildRepoFile(repoBaseUrl: string) {
  return `[synforge]
name=Synforge Managed Repository
baseurl=${repoBaseUrl}
enabled=1
gpgcheck=0
repo_gpgcheck=0
metadata_expire=30s`;
}

function buildRepoWriteCommands(repoFileContents: string): Record<ShellKind, string> {
  return {
    bash: `cat <<'EOF' | sudo tee /etc/yum.repos.d/synforge.repo >/dev/null\n${repoFileContents}\nEOF`,
    zsh: `cat <<'EOF' | sudo tee /etc/yum.repos.d/synforge.repo >/dev/null\n${repoFileContents}\nEOF`,
    fish: `begin\n${repoFileContents
      .split("\n")
      .map((line) => `  echo "${line.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`)
      .join("\n")}\nend | sudo tee /etc/yum.repos.d/synforge.repo >/dev/null`,
  };
}

function normalizeBaseUrl(baseUrl: string) {
  return baseUrl.replace(/\/+$/, "");
}
