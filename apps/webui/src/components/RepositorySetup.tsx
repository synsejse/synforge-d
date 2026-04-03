import { useEffect, useMemo, useState } from "react";
import api from "../lib/api";
import PageHeader from "./PageHeader";
import FaIcon from "./FaIcon";
import {
  faCopy,
  faFolderTree,
  faTerminal,
} from "@fortawesome/free-solid-svg-icons";

export default function RepositorySetup() {
  const [publicBaseUrl, setPublicBaseUrl] = useState("");
  const [repoHandle, setRepoHandle] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [copiedLabel, setCopiedLabel] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      try {
        setLoading(true);
        const [configRes, sessionRes] = await Promise.all([
          api.getConfig(),
          api.getSession(),
        ]);
        setPublicBaseUrl(normalizeBaseUrl(configRes.config.public_base_url));
        setRepoHandle(sessionRes.user.handle);
        setError(null);
      } catch (e) {
        setError(
          e instanceof Error ? e.message : "Failed to load repository setup",
        );
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

  const repoFileContents = useMemo(
    () => buildRepoFile(repoBaseUrl, repoHandle),
    [repoBaseUrl, repoHandle],
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

  if (loading) {
    return <div className="text-zinc-400">Loading repository setup…</div>;
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
        eyebrow="Repository Access"
        title="Add Repo To Fedora"
        description="Repo file and basic DNF usage."
        actions={[
          {
            href: "/repository/",
            label: "Browse Repository",
            icon: faFolderTree,
          },
        ]}
      />

      <section className="border border-zinc-800 bg-black p-6">
        <div>
          <h2 className="text-xl font-semibold text-white">Repo File</h2>
          <p className="mt-2 text-sm text-zinc-400">
            Copy this into{" "}
            <span className="font-mono text-zinc-200">
              /etc/yum.repos.d/synforge.repo
            </span>
            . The current account handle is already filled into{" "}
            <span className="font-mono text-zinc-200">username</span>; replace{" "}
            <span className="font-mono text-zinc-200">&lt;password&gt;</span>{" "}
            with that account's password or another user that has the{" "}
            <span className="font-mono text-zinc-200">repo</span> permission.
          </p>
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

      <section className="grid gap-4">
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
              onCopy={() =>
                copy(
                  "usage-command",
                  `sudo dnf clean all\nsudo dnf makecache\n${installCommand}`,
                )
              }
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
        <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">
          {label}
        </div>
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

function buildRepoFile(repoBaseUrl: string, repoHandle: string) {
  return `[synforge]
name=Synforge Managed Repository
baseurl=${repoBaseUrl}
username=${repoHandle || "<handle>"}
password=<password>
enabled=1
gpgcheck=0
repo_gpgcheck=0
metadata_expire=30s`;
}

function normalizeBaseUrl(baseUrl: string) {
  return baseUrl.replace(/\/+$/, "");
}
