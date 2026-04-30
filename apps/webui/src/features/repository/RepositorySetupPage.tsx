import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { faCopy, faFolderTree } from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { queryKeys } from "../../lib/query-keys";
import ErrorMessage from "../../components/common/ErrorMessage";
import { useSession } from "../../components/common/SessionProvider";
import LoadingBlock from "../../components/ui/LoadingBlock";
import FaIcon from "../../components/ui/FaIcon";
import Button from "../../components/ui/Button";

const REPO_PUBLIC_KEY_NAME = "gpg.key";
const INSTALL_COMMAND = "sudo dnf install <package-name>";

function RepositorySetup() {
  const { session } = useSession();
  const [copiedLabel, setCopiedLabel] = useState<string | null>(null);

  const setupQuery = useQuery({
    queryKey: queryKeys.repository.setup(),
    queryFn: async () => {
      const [config, signing] = await Promise.all([
        api.getConfig(),
        api.getRepoSigningStatus(),
      ]);
      return {
        publicBaseUrl: normalizeBaseUrl(config.config.public_base_url),
        signingEnabled: signing.status.enabled,
      };
    },
  });

  const repoHandle = session?.user.handle ?? "";

  const repoRootUrl = useMemo(() => {
    if (!setupQuery.data?.publicBaseUrl) return "";
    return `${setupQuery.data.publicBaseUrl}/repo`;
  }, [setupQuery.data?.publicBaseUrl]);

  const repoBaseUrl = useMemo(() => {
    if (!repoRootUrl) return "";
    return `${repoRootUrl}/fedora/$releasever`;
  }, [repoRootUrl]);

  const repoFileContents = useMemo(
    () =>
      buildRepoFile(
        repoRootUrl,
        repoBaseUrl,
        repoHandle,
        setupQuery.data?.signingEnabled ?? false,
        REPO_PUBLIC_KEY_NAME,
      ),
    [repoRootUrl, repoBaseUrl, repoHandle, setupQuery.data?.signingEnabled],
  );

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

  if (setupQuery.isPending) {
    return <LoadingBlock label="Loading repository setup…" lines={3} />;
  }

  if (setupQuery.error) {
    return (
      <ErrorMessage
        message={
          setupQuery.error instanceof Error
            ? setupQuery.error.message
            : "Failed to load repository setup"
        }
      />
    );
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="border-4 border-[var(--theme-accent-orange)] bg-black p-6">
        <div className="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
          <div className="min-w-0 flex-1">
            <div className="font-mono text-xs font-bold uppercase tracking-[0.3em] text-[var(--theme-accent-orange)]">
              REPOSITORY_ACCESS
            </div>
            <h1 className="mt-2 font-mono text-3xl font-bold uppercase text-white">
              Add Repo To Fedora
            </h1>
            <p className="mt-2 text-sm text-zinc-400">
              Fedora repo file and basic DNF usage.
            </p>
          </div>
          <div className="flex gap-3">
            <a href="/repository/">
              <Button variant="secondary" size="md">
                <FaIcon icon={faFolderTree} className="mr-2" />
                Browse Repository
              </Button>
            </a>
          </div>
        </div>
      </div>

      {/* Repo File */}
      <div className="border-2 border-white bg-black">
        <div className="border-b-2 border-zinc-800 bg-black px-6 py-5">
          <h2 className="font-mono text-lg font-bold uppercase text-white">
            Repo File
          </h2>
          <p className="mt-2 text-sm text-zinc-400">
            Copy this into{" "}
            <span className="font-mono text-[var(--theme-accent-lime)]">
              /etc/yum.repos.d/synforge.repo
            </span>
            . The current account handle is already filled into{" "}
            <span className="font-mono text-[var(--theme-accent-lime)]">username</span>; replace{" "}
            <span className="font-mono text-[var(--theme-accent-lime)]">&lt;password&gt;</span>{" "}
            with that account's password or another user that has the{" "}
            <span className="font-mono text-[var(--theme-accent-lime)]">repo</span> permission.
            Repository signing settings are reflected automatically in the snippet below.
          </p>
        </div>

        <div className="border-2 border-[var(--theme-accent-lime)] bg-black m-6">
          <div className="flex items-center justify-between border-b-2 border-zinc-800 bg-zinc-950 px-4 py-3">
            <div className="font-mono text-xs font-bold uppercase tracking-[0.2em] text-[var(--theme-accent-lime)]">
              synforge.repo
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => copy("repo-file", repoFileContents)}
            >
              <FaIcon icon={faCopy} className="mr-2" />
              {copiedLabel === "repo-file" ? "Copied!" : "Copy"}
            </Button>
          </div>
          <div className="overflow-x-auto">
            <pre className="p-6 font-mono text-sm leading-7 text-zinc-100">
              {repoFileContents.split('\n').map((line, i) => {
                if (line.startsWith('[')) {
                  return <div key={i} className="text-[var(--theme-accent-orange)]">{line}</div>;
                }
                if (line.includes('=')) {
                  const [key, ...rest] = line.split('=');
                  const value = rest.join('=');
                  return (
                    <div key={i}>
                      <span className="text-cyan-400">{key}</span>
                      <span className="text-zinc-500">=</span>
                      <span className="text-[var(--theme-accent-lime)]">{value}</span>
                    </div>
                  );
                }
                return <div key={i}>{line}</div>;
              })}
            </pre>
          </div>
        </div>
      </div>

      {/* Usage Commands */}
      <div className="border-2 border-white bg-black">
        <div className="border-b-2 border-zinc-800 bg-black px-6 py-5">
          <h2 className="font-mono text-lg font-bold uppercase text-white">
            Use Repo
          </h2>
          <p className="mt-2 text-sm text-zinc-400">
            Commands to refresh DNF cache and install packages from this repository.
          </p>
        </div>

        <div className="border-2 border-[var(--theme-terminal-green)] bg-black m-6">
          <div className="flex items-center justify-between border-b-2 border-zinc-800 bg-zinc-950 px-4 py-3">
            <div className="font-mono text-xs font-bold uppercase tracking-[0.2em] text-[var(--theme-terminal-green)]">
              usage.sh
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() =>
                copy(
                  "usage-command",
                  `sudo dnf clean all\nsudo dnf makecache\n${INSTALL_COMMAND}`,
                )
              }
            >
              <FaIcon icon={faCopy} className="mr-2" />
              {copiedLabel === "usage-command" ? "Copied!" : "Copy"}
            </Button>
          </div>
          <div className="overflow-x-auto">
            <pre className="p-6 font-mono text-sm leading-7">
              <div><span className="text-[var(--theme-accent-orange)]">$</span> <span className="text-cyan-400">sudo</span> <span className="text-zinc-100">dnf clean all</span></div>
              <div><span className="text-[var(--theme-accent-orange)]">$</span> <span className="text-cyan-400">sudo</span> <span className="text-zinc-100">dnf makecache</span></div>
              <div><span className="text-[var(--theme-accent-orange)]">$</span> <span className="text-cyan-400">sudo</span> <span className="text-zinc-100">dnf install</span> <span className="text-[var(--theme-terminal-green)]">&lt;package-name&gt;</span></div>
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
}

function buildRepoFile(
  repoRootUrl: string,
  repoBaseUrl: string,
  repoHandle: string,
  repoSigningEnabled: boolean,
  repoPublicKeyName: string,
) {
  const signingLines = repoSigningEnabled
    ? `gpgcheck=1
repo_gpgcheck=0
gpgkey=${repoRootUrl}/${encodeURIComponent(repoPublicKeyName)}`
    : `gpgcheck=0
repo_gpgcheck=0`;
  return `[synforge]
name=Synforge Managed Repository
baseurl=${repoBaseUrl}
username=${repoHandle || "<handle>"}
password=<password>
enabled=1
${signingLines}
metadata_expire=30s`;
}

function normalizeBaseUrl(baseUrl: string) {
  return baseUrl.replace(/\/+$/, "");
}

export default function RepositorySetupPage() {
  return <RepositorySetup />;
}
