import { useEffect, useMemo, useState } from "react";
import api from "../../lib/api";
import ErrorMessage from "../common/ErrorMessage";
import LoadingBlock from "../ui/LoadingBlock";
import PageHeader from "../ui/PageHeader";
import FaIcon from "../ui/FaIcon";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";
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
    return <LoadingBlock label="Loading repository setup…" lines={3} />;
  }

  if (error) {
    return <ErrorMessage message={error} />;
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
            language="ini"
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
              language="bash"
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
  language,
  copied,
  onCopy,
}: {
  label: string;
  value: string;
  language: string;
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
      <SyntaxHighlighter
        language={language}
        style={oneDark}
        customStyle={{
          margin: 0,
          padding: "1rem",
          background: "transparent",
          fontSize: "0.875rem",
          lineHeight: "1.5rem",
          overflowX: "auto",
        }}
        codeTagProps={{
          style: {
            fontFamily:
              'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace',
          },
        }}
      >
        {value}
      </SyntaxHighlighter>
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
