import { useRef, useState, type ChangeEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { faCheckCircle, faKey, faTrash } from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { queryKeys } from "../../lib/query-keys";
import type {
  RepoSigningReconcileMode,
  RepoSigningReconcileProgressView,
} from "../../lib/types";
import PageRoot from "../../components/common/PageRoot";
import ErrorMessage from "../../components/common/ErrorMessage";
import Button from "../../components/ui/Button";
import FaIcon from "../../components/ui/FaIcon";
import LoadingBlock from "../../components/ui/LoadingBlock";
import PageHeader from "../../components/ui/PageHeader";
import ProgressOverlayDialog from "../../components/ui/ProgressOverlayDialog";

function downloadBlob(filename: string, contents: string, type: string) {
  const blob = new Blob([contents], { type });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

function progressPercent(operation: RepoSigningReconcileProgressView): number {
  if (operation.state !== "running") return 100;
  if (operation.total_artifacts === 0) return 100;
  return Math.min(
    100,
    Math.round((operation.processed_artifacts / operation.total_artifacts) * 100),
  );
}

function progressDetail(operation: RepoSigningReconcileProgressView): string {
  const counts = `${operation.processed_artifacts}/${operation.total_artifacts} artifacts · ${operation.failed_artifacts} failed`;
  if (operation.state === "completed") {
    return operation.failed_artifacts > 0
      ? `Finished with failures · ${counts}`
      : `Finished · ${counts}`;
  }
  if (operation.state === "failed") {
    return operation.message ? `Failed · ${operation.message}` : `Failed · ${counts}`;
  }
  return counts;
}

function Signing() {
  const queryClient = useQueryClient();
  const importFileRef = useRef<HTMLInputElement | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [overlayOpen, setOverlayOpen] = useState(false);
  const [reconcileMode, setReconcileMode] =
    useState<RepoSigningReconcileMode | null>(null);

  const statusQuery = useQuery({
    queryKey: queryKeys.signing.status(),
    queryFn: () => api.getRepoSigningStatus(),
  });
  const status = statusQuery.data?.status;
  const enabled = status?.enabled ?? false;
  const keyActionsLocked = enabled;

  const progressQuery = useQuery({
    queryKey: queryKeys.signing.reconcileProgress(),
    queryFn: () => api.getRepoSigningReconcileProgress(),
    enabled: overlayOpen,
    refetchInterval: overlayOpen ? 500 : false,
  });

  const refreshStatus = () =>
    queryClient.invalidateQueries({ queryKey: queryKeys.signing.status() });

  const toggleMutation = useMutation({
    mutationFn: async (nextEnabled: boolean) => {
      const response = await api.updateRepoSigningConfig({ enabled: nextEnabled });
      if (nextEnabled) {
        await api.testRepoSigning();
      }
      return { response, nextEnabled };
    },
    onMutate: (nextEnabled) => {
      setReconcileMode(nextEnabled ? "sign" : "unsign");
      setOverlayOpen(true);
    },
    onSuccess: async ({ nextEnabled }) => {
      await refreshStatus();
      setMessage(
        nextEnabled
          ? "Repository signing enabled. Signing test passed."
          : "Repository signing disabled.",
      );
      setError(null);
      setOverlayOpen(false);
    },
    onError: (err) => {
      setError(err instanceof Error ? err.message : "Failed to update signing state");
    },
  });

  const generateMutation = useMutation({
    mutationFn: () => api.generateRepoSigningKey(),
    onSuccess: async (response) => {
      await refreshStatus();
      setMessage(`Generated managed key ${response.key_id}.`);
      setError(null);
    },
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to generate signing key"),
  });

  const importMutation = useMutation({
    mutationFn: (armoredPrivateKey: string) =>
      api.importRepoSigningKey({ armored_private_key: armoredPrivateKey }),
    onSuccess: async (response, _, _ctx) => {
      await refreshStatus();
      setMessage(`Imported signing key ${response.key_id}.`);
      setError(null);
    },
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to import signing key"),
  });

  const exportPrivateMutation = useMutation({
    mutationFn: () => api.exportRepoSigningKey(),
    onSuccess: (response) => {
      const safeKeyId = response.key_id.replace(/[^a-zA-Z0-9._-]/g, "_");
      const filename = `synforge-signing-key-${safeKeyId || "export"}.asc`;
      const contents = response.armored_private_key.endsWith("\n")
        ? response.armored_private_key
        : `${response.armored_private_key}\n`;
      downloadBlob(filename, contents, "application/pgp-keys");
      setMessage(`Exported key ${response.key_id} to ${filename}.`);
      setError(null);
    },
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to export signing key"),
  });

  const exportPublicMutation = useMutation({
    mutationFn: () => api.exportRepoSigningPublicKey(),
    onSuccess: (response) => {
      const safeKeyName = response.public_key_name.replace(/[^a-zA-Z0-9._-]/g, "_");
      const filename = safeKeyName || "synforge-public-key.asc";
      const contents = response.armored_public_key.endsWith("\n")
        ? response.armored_public_key
        : `${response.armored_public_key}\n`;
      downloadBlob(filename, contents, "application/pgp-keys");
      setMessage(`Exported public key ${response.key_id} to ${filename}.`);
      setError(null);
    },
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to export public key"),
  });

  const deleteMutation = useMutation({
    mutationFn: () => api.removeRepoSigningKey(),
    onSuccess: async () => {
      await refreshStatus();
      setMessage("Signing key deleted.");
      setError(null);
    },
    onError: (err) =>
      setError(err instanceof Error ? err.message : "Failed to delete signing key"),
  });

  function handleToggleSigning() {
    toggleMutation.mutate(!enabled);
  }

  function handleGenerateKey() {
    if (keyActionsLocked) {
      setError("Disable signing before generating a new key.");
      return;
    }
    generateMutation.mutate();
  }

  async function handleImportFile(event: ChangeEvent<HTMLInputElement>) {
    if (keyActionsLocked) {
      setError("Disable signing before importing a key.");
      event.target.value = "";
      return;
    }
    const file = event.target.files?.[0];
    if (!file) return;
    try {
      const armoredPrivateKey = await file.text();
      importMutation.mutate(armoredPrivateKey);
    } finally {
      event.target.value = "";
    }
  }

  if (statusQuery.isPending) {
    return <LoadingBlock label="Loading signing status…" lines={4} />;
  }

  if (statusQuery.error || !status) {
    return (
      <ErrorMessage
        message={
          statusQuery.error instanceof Error
            ? statusQuery.error.message
            : "Failed to load signing status"
        }
      />
    );
  }

  const operation = progressQuery.data?.operation ?? null;
  const showOperation =
    operation && (!reconcileMode || operation.mode === reconcileMode);
  const overlayTitle = reconcileMode
    ? reconcileMode === "sign"
      ? "Signing existing artifacts"
      : "Unsigning existing artifacts"
    : "Processing signing state";
  const overlayProgress = showOperation ? progressPercent(operation) : 0;
  const overlayDetail = showOperation
    ? progressDetail(operation)
    : "Preparing artifact reconciliation…";

  return (
    <div className="space-y-8">
      <PageHeader
        eyebrow="REPOSITORY_SIGNING"
        title="GPG Signing"
        description="Generate or import a private key, then toggle repository signing."
        color="orange"
        actions={[
          { href: "/repository/use/", label: "Repo Setup", icon: faCheckCircle },
        ]}
      />

      {error ? <ErrorMessage message={error} /> : null}
      {message ? (
        <div className="border-2 border-[var(--theme-terminal-green)] bg-black px-4 py-3 text-sm text-zinc-100">
          {message}
        </div>
      ) : null}

      <section className="grid gap-6 xl:grid-cols-2">
        <article className="border-2 border-white bg-black p-6">
          <h2 className="font-mono text-sm font-bold uppercase tracking-[0.2em] text-white">
            Current_Status
          </h2>
          <dl className="mt-4 grid gap-px bg-zinc-800">
            <StatusRow label="Enabled" value={status.enabled ? "yes" : "no"} />
            <StatusRow
              label="Configured key id"
              value={status.configured_key_id || "-"}
            />
            <StatusRow
              label="Key present"
              value={status.key_present ? "yes" : "no"}
            />
            <StatusRow
              label="Fingerprint"
              value={status.active_fingerprint || "-"}
            />
            <StatusRow
              label="Public key path"
              value={status.repo_public_key_path || "-"}
            />
          </dl>
        </article>

        <article className="border-2 border-white bg-black p-6">
          <h2 className="font-mono text-sm font-bold uppercase tracking-[0.2em] text-white">
            Signing_Actions
          </h2>
          <p className="mt-3 text-sm text-zinc-400">
            Public key filename is fixed to <code>gpg.key</code>. Key ID is
            always derived from the active private key.
          </p>
          <div className="mt-5 space-y-5">
            <div>
              <p className="mb-2 font-mono text-xs font-bold uppercase tracking-[0.16em] text-zinc-400">
                Signing State
              </p>
              <div className="flex flex-wrap gap-3">
                <Button
                  type="button"
                  variant={enabled ? "secondary" : "primary"}
                  size="md"
                  onClick={handleToggleSigning}
                  disabled={toggleMutation.isPending || (!enabled && !status.key_present)}
                >
                  <FaIcon icon={faCheckCircle} className="mr-2" />
                  {toggleMutation.isPending
                    ? "Updating…"
                    : enabled
                      ? "Disable Signing"
                      : "Enable Signing"}
                </Button>
              </div>
              {!enabled && !status.key_present ? (
                <p className="mt-2 font-mono text-xs text-zinc-500">
                  Generate or import a key before enabling signing.
                </p>
              ) : null}
            </div>

            <div>
              <p className="mb-2 font-mono text-xs font-bold uppercase tracking-[0.16em] text-zinc-400">
                Key Lifecycle
              </p>
              <div className="flex flex-wrap gap-3">
                <Button
                  type="button"
                  variant="secondary"
                  size="md"
                  onClick={handleGenerateKey}
                  disabled={generateMutation.isPending || keyActionsLocked}
                >
                  <FaIcon icon={faKey} className="mr-2" />
                  {generateMutation.isPending ? "Generating…" : "Generate Key"}
                </Button>
                <Button
                  type="button"
                  variant="secondary"
                  size="md"
                  onClick={() => importFileRef.current?.click()}
                  disabled={importMutation.isPending || keyActionsLocked}
                >
                  <FaIcon icon={faKey} className="mr-2" />
                  {importMutation.isPending ? "Importing…" : "Import Key File"}
                </Button>
                <Button
                  type="button"
                  variant="danger"
                  size="md"
                  onClick={() => deleteMutation.mutate()}
                  disabled={deleteMutation.isPending || enabled || !status.key_present}
                >
                  <FaIcon icon={faTrash} className="mr-2" />
                  {deleteMutation.isPending ? "Deleting…" : "Delete Key"}
                </Button>
              </div>
              {enabled ? (
                <p className="mt-2 font-mono text-xs text-zinc-500">
                  Disable signing before generating, importing, or deleting the key.
                </p>
              ) : null}
            </div>

            <div>
              <p className="mb-2 font-mono text-xs font-bold uppercase tracking-[0.16em] text-zinc-400">
                Backup
              </p>
              <div className="flex flex-wrap gap-3">
                <Button
                  type="button"
                  variant="secondary"
                  size="md"
                  onClick={() => exportPublicMutation.mutate()}
                  disabled={exportPublicMutation.isPending || !status.key_present}
                >
                  <FaIcon icon={faKey} className="mr-2" />
                  {exportPublicMutation.isPending
                    ? "Exporting…"
                    : "Export Public Key"}
                </Button>
                {status.can_export_private_key ? (
                  <Button
                    type="button"
                    variant="secondary"
                    size="md"
                    onClick={() => exportPrivateMutation.mutate()}
                    disabled={exportPrivateMutation.isPending || !status.key_present}
                  >
                    <FaIcon icon={faKey} className="mr-2" />
                    {exportPrivateMutation.isPending
                      ? "Exporting…"
                      : "Export Private Key"}
                  </Button>
                ) : (
                  <span className="font-mono text-xs text-zinc-500">
                    Export is restricted to bootstrap admin.
                  </span>
                )}
              </div>
            </div>
          </div>
          <input
            ref={importFileRef}
            type="file"
            accept=".asc,.key,.pgp,.gpg,text/plain"
            className="hidden"
            onChange={handleImportFile}
          />
        </article>
      </section>

      <ProgressOverlayDialog
        open={overlayOpen}
        title={overlayTitle}
        detail={overlayDetail}
        progress={overlayProgress}
        onClose={() => setOverlayOpen(false)}
        closeDisabled={toggleMutation.isPending}
      />
    </div>
  );
}

function StatusRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid gap-1 bg-black px-4 py-3 text-xs sm:grid-cols-[minmax(0,180px)_1fr] sm:gap-4">
      <dt className="font-mono font-bold uppercase tracking-[0.16em] text-zinc-400">
        {label}
      </dt>
      <dd className="break-all font-mono text-zinc-100 sm:truncate">{value}</dd>
    </div>
  );
}

export default function SigningPage() {
  return (
    <PageRoot>
      <Signing />
    </PageRoot>
  );
}
