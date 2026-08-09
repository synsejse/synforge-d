import type { ReactNode } from "react";
import { formatMockChroots } from "../../../../lib/utils";
import type { AddPackageFormState } from "./form-state";

export default function ReviewSection({ form }: { form: AddPackageFormState }) {
  return (
    <div className="space-y-4">
      <ReviewGroup title="Source">
        <ReviewRow label="Synforge name" value={form.name} />
        <ReviewRow label="Repository" value={form.repoUrl} />
        <ReviewRow label="Spec file" value={form.specPath} />
      </ReviewGroup>

      <ReviewGroup title="Targets">
        <ReviewRow
          label="Mock chroots"
          value={formatMockChroots(form.mockChroots, "None selected")}
        />
        <ReviewRow label="Package enabled" value={yesNo(form.enabled)} />
        <ReviewRow label="Source polling" value={yesNo(form.poll)} />
        <ReviewRow label="Publish SRPM" value={yesNo(form.publishSrpm)} />
        <ReviewRow
          label="Publish debug packages"
          value={yesNo(form.publishDebuginfo)}
        />
      </ReviewGroup>

      <ReviewGroup title="Build">
        <ReviewRow
          label="Compiler cache"
          value={
            form.ccacheEnabled
              ? form.ccacheMaxSizeMb.trim()
                ? `Enabled · ${form.ccacheMaxSizeMb} MB per target`
                : "Enabled · Mock default size"
              : "Disabled"
          }
        />
        <ReviewRow
          label="Timeout"
          value={`${form.buildTimeoutSeconds} seconds`}
        />
        {form.poll ? (
          <ReviewRow
            label="Poll interval"
            value={`${form.pollIntervalSeconds} seconds`}
          />
        ) : null}
        <ReviewRow label="History retained" value={form.packageHistoryCount} />
        <ReviewRow
          label="CPU limit"
          value={
            form.cpuLimitEnabled
              ? `${form.cpuLimitCores} cores`
              : "Unlimited"
          }
        />
        <ReviewRow
          label="Memory limit"
          value={
            form.memoryLimitEnabled
              ? `${form.memoryLimitMb} MB`
              : "Unlimited"
          }
        />
        <ReviewRow label="Network access" value={yesNo(form.networkAccess)} />
        <ReviewRow
          label="Environment entries"
          value={String(
            form.buildEnv.split("\n").filter((line) => line.trim()).length,
          )}
        />
      </ReviewGroup>
    </div>
  );
}

function ReviewGroup({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="border border-edge bg-surface-alt p-5">
      <h3 className="font-mono text-xs font-bold uppercase tracking-[0.18em] text-accent-lime">
        {title}
      </h3>
      <dl className="mt-4 grid gap-x-6 gap-y-3 sm:grid-cols-2">{children}</dl>
    </section>
  );
}

function ReviewRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0">
      <dt className="font-mono text-xs uppercase tracking-[0.12em] text-soft">
        {label}
      </dt>
      <dd className="mt-1 break-words font-mono text-sm text-strong">{value}</dd>
    </div>
  );
}

function yesNo(value: boolean): string {
  return value ? "Yes" : "No";
}
