import { useRef, useState, type ChangeEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  faCheckCircle,
} from "@fortawesome/free-solid-svg-icons";
import api from "../../lib/api";
import { signingQueries } from "../../lib/queries";
import type {
  RepoSigningReconcileMode,
  RepoSigningReconcileProgressView,
} from "../../lib/types";
import ErrorMessage from "../../components/common/error-message";
import LoadingBlock from "../../components/ui/loading-block";
import PageHeader from "../../components/ui/page-header";
import ProgressOverlayDialog from "../../components/ui/progress-overlay-dialog";
import SigningControls from "./signing-controls";

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

  const statusQuery = useQuery(signingQueries.status());
  const status = statusQuery.data?.status;
  const enabled = status?.enabled ?? false;
  const keyActionsLocked = enabled;

  const progressQuery = useQuery({
    ...signingQueries.reconcileProgress(),
    enabled: overlayOpen,
    refetchInterval: overlayOpen ? 500 : false,
  });

  const refreshStatus = () =>
    queryClient.invalidateQueries({ queryKey: signingQueries.status().queryKey });

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
    return (
      <div className="space-y-6">
        <PageHeader
          title="GPG Signing"
          description="Generate or import a private key, then toggle repository signing."
          color="orange"
          actions={[
            { to: "/repository/use", label: "Repo Setup", icon: faCheckCircle },
          ]}
        />
        <LoadingBlock label="Loading signing status…" lines={3} />
      </div>
    );
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
    <div className="space-y-6">
      <PageHeader
        title="GPG Signing"
        description="Generate or import a private key, then toggle repository signing."
        color="orange"
        actions={[
          { to: "/repository/use", label: "Repo Setup", icon: faCheckCircle },
        ]}
      />

      {error ? <ErrorMessage message={error} /> : null}
      {message ? (
        <div className="border border-success bg-black px-4 py-3 text-sm text-strong">
          {message}
        </div>
      ) : null}

      <SigningControls
        status={status}
        importFileRef={importFileRef}
        onToggle={handleToggleSigning}
        onGenerate={handleGenerateKey}
        onImport={handleImportFile}
        onExportPublic={() => exportPublicMutation.mutate()}
        onExportPrivate={() => exportPrivateMutation.mutate()}
        onDelete={() => deleteMutation.mutate()}
        pending={{
          toggle: toggleMutation.isPending,
          generate: generateMutation.isPending,
          import: importMutation.isPending,
          exportPublic: exportPublicMutation.isPending,
          exportPrivate: exportPrivateMutation.isPending,
          delete: deleteMutation.isPending,
        }}
      />

      <ProgressOverlayDialog
        open={overlayOpen}
        title={overlayTitle}
        progress={overlayProgress}
        onClose={() => setOverlayOpen(false)}
        closeDisabled={toggleMutation.isPending}
      >
        <p className="font-mono text-xs text-soft">{overlayDetail}</p>
      </ProgressOverlayDialog>
    </div>
  );
}

export default function SigningPage() {
  return <Signing />;
}
