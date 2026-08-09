import type { ChangeEventHandler, RefObject } from "react";
import {
  faArrowUpFromBracket,
  faCheckCircle,
  faKey,
  faLock,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import type { RepoSigningStatusView } from "../../lib/types";
import Button from "../../components/ui/button";
import FaIcon from "../../components/ui/fa-icon";
import MetaPair from "../../components/ui/meta-pair";
import StatusPill from "../../components/ui/status-pill";

interface SigningControlsProps {
  status: RepoSigningStatusView;
  importFileRef: RefObject<HTMLInputElement | null>;
  onToggle: () => void;
  onGenerate: () => void;
  onImport: ChangeEventHandler<HTMLInputElement>;
  onExportPublic: () => void;
  onExportPrivate: () => void;
  onDelete: () => void;
  pending: {
    toggle: boolean;
    generate: boolean;
    import: boolean;
    exportPublic: boolean;
    exportPrivate: boolean;
    delete: boolean;
  };
}

export default function SigningControls({
  status,
  importFileRef,
  onToggle,
  onGenerate,
  onImport,
  onExportPublic,
  onExportPrivate,
  onDelete,
  pending,
}: SigningControlsProps) {
  const enabled = status.enabled;
  const keyActionsLocked = enabled;

  return (
    <>
      <section
        aria-label="Signing status"
        className="flex flex-wrap items-start gap-x-6 gap-y-3 border border-edge bg-black px-4 py-3 sm:px-5"
      >
        <div className="self-center">
          <StatusPill status={enabled ? "enabled" : "disabled"} />
        </div>
        <MetaPair label="Key">
          {status.key_present ? (
            <span className="text-xs text-strong">present</span>
          ) : (
            <span className="text-xs text-soft">none</span>
          )}
        </MetaPair>
        <MetaPair label="Key id">
          <span className="break-all font-mono text-xs text-strong">
            {status.configured_key_id || (
              <em className="not-italic text-soft">unset</em>
            )}
          </span>
        </MetaPair>
        <MetaPair label="Fingerprint">
          <span className="break-all font-mono text-xs text-strong">
            {status.active_fingerprint || (
              <em className="not-italic text-soft">none</em>
            )}
          </span>
        </MetaPair>
        <MetaPair label="Public key path">
          <span className="break-all font-mono text-xs text-soft">
            {status.repo_public_key_path || "—"}
          </span>
        </MetaPair>
      </section>

      <section className="border border-edge bg-black">
        <div className="flex flex-wrap items-center gap-2 px-4 py-3 sm:px-5">
          <Button
            variant={enabled ? "ghost" : "primary"}
            size="sm"
            onClick={onToggle}
            loading={pending.toggle}
            disabled={!enabled && !status.key_present}
          >
            {pending.toggle ? null : <FaIcon icon={faCheckCircle} />}
            {enabled ? "Disable Signing" : "Enable Signing"}
          </Button>

          <span className="mx-1 hidden h-6 w-px bg-edge sm:inline-block" />

          <Button
            variant="ghost"
            size="sm"
            onClick={onGenerate}
            loading={pending.generate}
            disabled={keyActionsLocked}
          >
            {pending.generate ? null : (
              <FaIcon icon={keyActionsLocked ? faLock : faKey} />
            )}
            Generate
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => importFileRef.current?.click()}
            loading={pending.import}
            disabled={keyActionsLocked}
          >
            {pending.import ? null : (
              <FaIcon icon={keyActionsLocked ? faLock : faKey} />
            )}
            Import
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={onExportPublic}
            loading={pending.exportPublic}
            disabled={!status.key_present}
          >
            {pending.exportPublic ? null : <FaIcon icon={faArrowUpFromBracket} />}
            Export Public
          </Button>
          {status.can_export_private_key ? (
            <Button
              variant="ghost"
              size="sm"
              onClick={onExportPrivate}
              loading={pending.exportPrivate}
              disabled={!status.key_present}
            >
              {pending.exportPrivate ? null : (
                <FaIcon icon={faArrowUpFromBracket} />
              )}
              Export Private
            </Button>
          ) : null}

          <span className="mx-1 hidden h-6 w-px bg-edge sm:inline-block" />

          <Button
            variant="danger"
            size="sm"
            onClick={onDelete}
            loading={pending.delete}
            disabled={enabled || !status.key_present}
          >
            {pending.delete ? null : (
              <FaIcon icon={enabled ? faLock : faTrash} />
            )}
            Delete Key
          </Button>
        </div>

        {!enabled && !status.key_present ? (
          <p className="border-t border-edge px-4 py-2 font-mono text-xs text-soft sm:px-5">
            Generate or import a key before enabling signing.
          </p>
        ) : null}
        {enabled ? (
          <p className="border-t border-edge px-4 py-2 font-mono text-xs text-soft sm:px-5">
            Disable signing before generating, importing, or deleting the key.
          </p>
        ) : null}
        {!status.can_export_private_key ? (
          <p className="border-t border-edge px-4 py-2 font-mono text-xs text-soft sm:px-5">
            Private key export is restricted to the bootstrap admin. Public key filename is fixed
            to <code> gpg.key</code>.
          </p>
        ) : null}

        <input
          ref={importFileRef}
          type="file"
          accept=".asc,.key,.pgp,.gpg,text/plain"
          className="hidden"
          onChange={onImport}
        />
      </section>
    </>
  );
}
