import { useEffect, useRef, useState, type ChangeEvent } from "react";
import { faCheckCircle, faKey, faTrash } from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import type { RepoSigningStatusView } from "../../lib/types";
import ErrorMessage from "../../components/common/ErrorMessage";
import Button from "../../components/ui/Button";
import FaIcon from "../../components/ui/FaIcon";
import LoadingBlock from "../../components/ui/LoadingBlock";
import PageHeader from "../../components/ui/PageHeader";
import ProgressOverlayDialog from "../../components/ui/ProgressOverlayDialog";

export default function Signing() {
  const importFileRef = useRef<HTMLInputElement | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<RepoSigningStatusView | null>(null);
  const [enabled, setEnabled] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [importing, setImporting] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [exportingPublic, setExportingPublic] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [overlayOpen, setOverlayOpen] = useState(false);
  const [overlayTitle, setOverlayTitle] = useState("Processing signing state");
  const [overlayDetail, setOverlayDetail] = useState("Preparing artifact reconciliation…");
  const [overlayProgress, setOverlayProgress] = useState(0);

  useEffect(() => {
    void loadStatus();
  }, []);

  async function loadStatus() {
    try {
      setLoading(true);
      const response = await api.getRepoSigningStatus();
      applyStatus(response.status);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load signing status");
    } finally {
      setLoading(false);
    }
  }

  function applyStatus(next: RepoSigningStatusView) {
    setStatus(next);
    setEnabled(next.enabled);
  }

  const keyActionsLocked = enabled;

  async function pollReconcileProgress() {
    const progress = await api.getRepoSigningReconcileProgress();
    const operation = progress.operation;
    if (!operation) {
      return;
    }
    setOverlayTitle(
      operation.mode === "sign"
        ? "Signing existing artifacts"
        : "Unsigning existing artifacts"
    );
    const percent =
      operation.total_artifacts === 0
        ? 100
        : Math.min(
            100,
            Math.round(
              (operation.processed_artifacts / operation.total_artifacts) * 100
            )
          );
    setOverlayProgress(percent);
    setOverlayDetail(
      `${operation.processed_artifacts}/${operation.total_artifacts} artifacts · ${operation.failed_artifacts} failed`
    );
  }

  async function handleToggleSigning() {
    const nextEnabled = !enabled;
    setSaving(true);
    setOverlayOpen(true);
    setOverlayProgress(0);
    setOverlayTitle(
      nextEnabled ? "Signing existing artifacts" : "Unsigning existing artifacts"
    );
    setOverlayDetail("Preparing artifact reconciliation…");
    const progressTicker = window.setInterval(() => {
      void pollReconcileProgress().catch(() => undefined);
    }, 500);
    try {
      await pollReconcileProgress().catch(() => undefined);
      const response = await api.updateRepoSigningConfig({ enabled: nextEnabled });
      await pollReconcileProgress().catch(() => undefined);
      applyStatus(response.status);
      if (nextEnabled) {
        await api.testRepoSigning();
        setMessage("Repository signing enabled. Signing test passed.");
      } else {
        setMessage("Repository signing disabled.");
      }
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to update signing state");
    } finally {
      window.clearInterval(progressTicker);
      setOverlayProgress(100);
      setSaving(false);
    }
  }

  async function handleGenerateKey() {
    if (keyActionsLocked) {
      setError("Disable signing before generating a new key.");
      return;
    }
    setGenerating(true);
    try {
      const response = await api.generateRepoSigningKey();
      applyStatus(response.status);
      setMessage(`Generated managed key ${response.key_id}.`);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to generate signing key");
    } finally {
      setGenerating(false);
    }
  }

  async function handleImportFile(event: ChangeEvent<HTMLInputElement>) {
    if (keyActionsLocked) {
      setError("Disable signing before importing a key.");
      event.target.value = "";
      return;
    }
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }
    setImporting(true);
    try {
      const armoredPrivateKey = await file.text();
      const response = await api.importRepoSigningKey({ armored_private_key: armoredPrivateKey });
      applyStatus(response.status);
      setMessage(`Imported signing key ${response.key_id} from ${file.name}.`);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to import signing key");
    } finally {
      event.target.value = "";
      setImporting(false);
    }
  }

  async function handleExportKey() {
    setExporting(true);
    try {
      const response = await api.exportRepoSigningKey();
      const safeKeyId = response.key_id.replace(/[^a-zA-Z0-9._-]/g, "_");
      const filename = `synforge-signing-key-${safeKeyId || "export"}.asc`;
      const contents = response.armored_private_key.endsWith("\n")
        ? response.armored_private_key
        : `${response.armored_private_key}\n`;
      const blob = new Blob([contents], { type: "application/pgp-keys" });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = filename;
      document.body.appendChild(anchor);
      anchor.click();
      anchor.remove();
      URL.revokeObjectURL(url);
      setMessage(`Exported key ${response.key_id} to ${filename}.`);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to export signing key");
    } finally {
      setExporting(false);
    }
  }

  async function handleExportPublicKey() {
    setExportingPublic(true);
    try {
      const response = await api.exportRepoSigningPublicKey();
      const safeKeyName = response.public_key_name.replace(/[^a-zA-Z0-9._-]/g, "_");
      const filename = safeKeyName || "synforge-public-key.asc";
      const contents = response.armored_public_key.endsWith("\n")
        ? response.armored_public_key
        : `${response.armored_public_key}\n`;
      const blob = new Blob([contents], { type: "application/pgp-keys" });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = filename;
      document.body.appendChild(anchor);
      anchor.click();
      anchor.remove();
      URL.revokeObjectURL(url);
      setMessage(`Exported public key ${response.key_id} to ${filename}.`);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to export public key");
    } finally {
      setExportingPublic(false);
    }
  }

  async function handleDeleteKey() {
    setDeleting(true);
    try {
      const response = await api.removeRepoSigningKey();
      applyStatus(response.status);
      setMessage("Signing key deleted.");
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to delete signing key");
    } finally {
      setDeleting(false);
    }
  }

  if (loading) {
    return <LoadingBlock label="Loading signing status…" lines={4} />;
  }

  if (error && !status) {
    return <ErrorMessage message={error} />;
  }

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
            <StatusRow label="Enabled" value={status?.enabled ? "yes" : "no"} />
            <StatusRow
              label="Configured key id"
              value={status?.configured_key_id || "-"}
            />
            <StatusRow
              label="Key present"
              value={status?.key_present ? "yes" : "no"}
            />
            <StatusRow
              label="Fingerprint"
              value={status?.active_fingerprint || "-"}
            />
            <StatusRow
              label="Public key path"
              value={status?.repo_public_key_path || "-"}
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
                  disabled={saving || (!enabled && !status?.key_present)}
                >
                  <FaIcon icon={faCheckCircle} className="mr-2" />
                  {saving ? "Updating…" : enabled ? "Disable Signing" : "Enable Signing"}
                </Button>
              </div>
              {!enabled && !status?.key_present ? (
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
                  disabled={generating || keyActionsLocked}
                >
                  <FaIcon icon={faKey} className="mr-2" />
                  {generating ? "Generating…" : "Generate Key"}
                </Button>
                <Button
                  type="button"
                  variant="secondary"
                  size="md"
                  onClick={() => importFileRef.current?.click()}
                  disabled={importing || keyActionsLocked}
                >
                  <FaIcon icon={faKey} className="mr-2" />
                  {importing ? "Importing…" : "Import Key File"}
                </Button>
                <Button
                  type="button"
                  variant="danger"
                  size="md"
                  onClick={handleDeleteKey}
                  disabled={deleting || enabled || !status?.key_present}
                >
                  <FaIcon icon={faTrash} className="mr-2" />
                  {deleting ? "Deleting…" : "Delete Key"}
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
                  onClick={handleExportPublicKey}
                  disabled={exportingPublic || !status?.key_present}
                >
                  <FaIcon icon={faKey} className="mr-2" />
                  {exportingPublic ? "Exporting…" : "Export Public Key"}
                </Button>
                {status?.can_export_private_key ? (
                  <Button
                    type="button"
                    variant="secondary"
                    size="md"
                    onClick={handleExportKey}
                    disabled={exporting || !status?.key_present}
                  >
                    <FaIcon icon={faKey} className="mr-2" />
                    {exporting ? "Exporting…" : "Export Private Key"}
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
        closeDisabled={saving}
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
