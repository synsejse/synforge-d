interface SourceBasicsSectionProps {
  ccacheEnabled: boolean;
  ccacheMaxSizeMb: string;
  enabled: boolean;
  name: string;
  networkAccess: boolean;
  poll: boolean;
  publishDebuginfo: boolean;
  publishSrpm: boolean;
  repoUrl: string;
  setCcacheEnabled: (value: boolean) => void;
  setCcacheMaxSizeMb: (value: string) => void;
  setEnabled: (value: boolean) => void;
  setName: (value: string) => void;
  setNetworkAccess: (value: boolean) => void;
  setPoll: (value: boolean) => void;
  setPublishDebuginfo: (value: boolean) => void;
  setPublishSrpm: (value: boolean) => void;
  setRepoUrl: (value: string) => void;
}

export default function SourceBasicsSection({
  ccacheEnabled,
  ccacheMaxSizeMb,
  enabled,
  name,
  networkAccess,
  poll,
  publishDebuginfo,
  publishSrpm,
  repoUrl,
  setCcacheEnabled,
  setCcacheMaxSizeMb,
  setEnabled,
  setName,
  setNetworkAccess,
  setPoll,
  setPublishDebuginfo,
  setPublishSrpm,
  setRepoUrl,
}: SourceBasicsSectionProps) {
  return (
    <>
      <label className="block">
        <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">
          Package name
        </span>
        <input
          type="text"
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder="mesa"
          required
          className="w-full border-2 border-edge-strong bg-surface-alt px-4 py-3 font-mono text-sm text-white placeholder:text-soft outline-none transition duration-100 ease-linear focus:border-accent-lime"
        />
      </label>

      <label className="block">
        <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.15em] text-muted">
          Git repository URL
        </span>
        <input
          type="url"
          value={repoUrl}
          onChange={(event) => setRepoUrl(event.target.value)}
          placeholder="https://github.com/example/repo.git"
          required
          className="w-full border-2 border-edge-strong bg-surface-alt px-4 py-3 font-mono text-sm text-white placeholder:text-soft outline-none transition duration-100 ease-linear focus:border-accent-lime"
        />
      </label>

      <div className="grid gap-4 md:grid-cols-2">
        <ToggleCard
          checked={ccacheEnabled}
          description="Reuse compiler cache per package and mock chroot."
          label="Shared ccache"
          onChange={setCcacheEnabled}
        />

        <label className="block border-2 border-edge-strong bg-surface-alt px-4 py-3">
          <span className="mb-2 block font-mono text-xs font-bold uppercase tracking-[0.1em] text-white">
            Shared ccache size (MB)
          </span>
          <input
            type="number"
            min={1}
            value={ccacheMaxSizeMb}
            onChange={(event) => setCcacheMaxSizeMb(event.target.value)}
            placeholder="Blank uses Mock default"
            className="w-full border-2 border-edge-strong bg-black px-4 py-3 font-mono text-sm text-white placeholder:text-soft outline-none transition duration-100 ease-linear focus:border-accent-lime"
          />
          <span className="mt-2 block text-xs text-soft">
            Applies per package and mock chroot.
          </span>
        </label>

        <ToggleCard
          checked={enabled}
          description="Allow new builds for this package."
          label="Enabled"
          onChange={setEnabled}
        />
        <ToggleCard
          checked={poll}
          description="Automatically watch the source for updates."
          label="Enable polling"
          onChange={setPoll}
        />
        <ToggleCard
          checked={publishSrpm}
          description="Keep source RPM publication enabled for this package."
          label="Publish SRPM"
          onChange={setPublishSrpm}
        />
        <ToggleCard
          checked={publishDebuginfo}
          description="Include debuginfo and debugsource RPMs in repository."
          label="Publish debug packages"
          onChange={setPublishDebuginfo}
        />
        <ToggleCard
          checked={networkAccess}
          className="md:col-span-2"
          description="Allow mock builds to access the network for packages that cannot build fully offline."
          label="Network access"
          onChange={setNetworkAccess}
        />
      </div>
    </>
  );
}

function ToggleCard({
  checked,
  className = "",
  description,
  label,
  onChange,
}: {
  checked: boolean;
  className?: string;
  description: string;
  label: string;
  onChange: (value: boolean) => void;
}) {
  return (
    <label
      className={`flex items-center justify-between border-2 border-edge-strong bg-surface-alt px-4 py-3 ${className}`}
    >
      <span>
        <span className="block font-mono text-xs font-bold uppercase tracking-[0.1em] text-white">
          {label}
        </span>
        <span className="mt-1 block text-xs text-soft">{description}</span>
      </span>
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
    </label>
  );
}
