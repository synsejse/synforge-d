import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { faCopy, faFolderTree } from "@fortawesome/free-solid-svg-icons";
import { repositoryQueries } from "../../lib/queries";
import ErrorMessage from "../../components/common/error-message";
import { useSession } from "../../components/common/session-provider";
import { SkeletonForm } from "../../components/ui/skeleton";
import FaIcon from "../../components/ui/fa-icon";
import Button from "../../components/ui/button";
import PageHeader from "../../components/ui/page-header";

const INSTALL_COMMAND = "sudo dnf install <package-name>";

function RepositorySetup() {
  const { session } = useSession();
  const [copiedLabel, setCopiedLabel] = useState<string | null>(null);

  const setupQuery = useQuery(repositoryQueries.setup());

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
        setupQuery.data?.publicKeyName ?? "gpg.key",
      ),
    [
      repoRootUrl,
      repoBaseUrl,
      repoHandle,
      setupQuery.data?.signingEnabled,
      setupQuery.data?.publicKeyName,
    ],
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
    return (
      <div className="space-y-6">
        <PageHeader
          title="Add Repo to Fedora"
          description="Fedora repo file and basic DNF usage."
          color="orange"
          actions={[
            { to: "/repository", label: "Browse Repository", icon: faFolderTree },
          ]}
        />
        <SkeletonForm sections={2} fieldsPerSection={2} />
      </div>
    );
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
    <div className="space-y-6">
      <PageHeader
        title="Add Repo to Fedora"
        description="Fedora repo file and basic DNF usage."
        color="orange"
        actions={[
          { to: "/repository", label: "Browse Repository", icon: faFolderTree },
        ]}
      />

      {/* Repo File — accented frame only, no double wrapper */}
      <section className="border-2 border-accent-lime bg-black">
        <div className="flex items-center gap-3 border-b-2 border-edge bg-surface-alt px-4 py-3">
          <div className="min-w-0 flex-1 truncate font-mono text-xs font-bold uppercase tracking-[0.18em] text-accent-lime">
            /etc/yum.repos.d/synforge.repo
          </div>
          <Button
            variant="ghost"
            size="sm"
            className="shrink-0"
            onClick={() => copy("repo-file", repoFileContents)}
            aria-label="Copy repo file"
          >
            <FaIcon icon={faCopy} />
            <span className="hidden sm:inline">
              {copiedLabel === "repo-file" ? "Copied!" : "Copy"}
            </span>
          </Button>
        </div>
        <div className="overflow-x-auto">
          <pre className="p-5 font-mono text-sm leading-7 text-strong">
            {repoFileContents.split("\n").map((line, i) => {
              if (line.startsWith("[")) {
                return (
                  <div key={i} className="text-accent-orange">
                    {line}
                  </div>
                );
              }
              if (line.includes("=")) {
                const [key, ...rest] = line.split("=");
                const value = rest.join("=");
                return (
                  <div key={i}>
                    <span className="text-accent-cyan">{key}</span>
                    <span className="text-soft">=</span>
                    <span className="text-accent-lime">{value}</span>
                  </div>
                );
              }
              return <div key={i}>{line}</div>;
            })}
          </pre>
        </div>
        <p className="border-t-2 border-edge bg-black px-4 py-3 text-xs text-soft">
          Replace{" "}
          <span className="font-mono text-accent-lime">&lt;password&gt;</span>{" "}
          with the account password (or any user with the{" "}
          <span className="font-mono text-accent-lime">repo</span>{" "}
          permission). Signing settings are reflected automatically.
        </p>
      </section>

      <section className="border-2 border-success bg-black">
        <div className="flex items-center gap-3 border-b-2 border-edge bg-surface-alt px-4 py-3">
          <div className="min-w-0 flex-1 truncate font-mono text-xs font-bold uppercase tracking-[0.18em] text-success">
            usage
          </div>
          <Button
            variant="ghost"
            size="sm"
            className="shrink-0"
            onClick={() =>
              copy(
                "usage-command",
                `sudo dnf clean all\nsudo dnf makecache\n${INSTALL_COMMAND}`,
              )
            }
            aria-label="Copy usage commands"
          >
            <FaIcon icon={faCopy} />
            <span className="hidden sm:inline">
              {copiedLabel === "usage-command" ? "Copied!" : "Copy"}
            </span>
          </Button>
        </div>
        <div className="overflow-x-auto">
          <pre className="p-5 font-mono text-sm leading-7">
            <div>
              <span className="text-accent-orange">$</span>{" "}
              <span className="text-accent-cyan">sudo</span>{" "}
              <span className="text-strong">dnf clean all</span>
            </div>
            <div>
              <span className="text-accent-orange">$</span>{" "}
              <span className="text-accent-cyan">sudo</span>{" "}
              <span className="text-strong">dnf makecache</span>
            </div>
            <div>
              <span className="text-accent-orange">$</span>{" "}
              <span className="text-accent-cyan">sudo</span>{" "}
              <span className="text-strong">dnf install</span>{" "}
              <span className="text-success">&lt;package-name&gt;</span>
            </div>
          </pre>
        </div>
      </section>
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

export default function RepositorySetupPage() {
  return <RepositorySetup />;
}
