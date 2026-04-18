use anyhow::Context;
use synforge_core::model::{WorkerAction, WorkerJobPayload};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct WorkerResourceLimits {
    pub cpu_limit_millicores: Option<u64>,
    pub memory_limit_mb: Option<u64>,
    pub nano_cpus: Option<i64>,
    pub memory_bytes: Option<i64>,
}

pub(super) fn worker_resource_limits(
    payload: &WorkerJobPayload,
) -> anyhow::Result<WorkerResourceLimits> {
    let WorkerAction::Build(build) = &payload.action else {
        return Ok(WorkerResourceLimits::default());
    };
    let cpu_limit_millicores = build
        .package
        .cpu_limit_millicores
        .filter(|value| *value > 0);
    let memory_limit_mb = build.package.memory_limit_mb.filter(|value| *value > 0);
    let nano_cpus = cpu_limit_millicores
        .map(|value| {
            value
                .checked_mul(1_000_000)
                .ok_or_else(|| anyhow::anyhow!("cpu limit is too large: {}", value))
        })
        .transpose()?
        .map(i64::try_from)
        .transpose()
        .context("cpu limit exceeds i64 range")?;
    let memory_bytes = memory_limit_mb
        .map(|value| {
            value
                .checked_mul(1024 * 1024)
                .ok_or_else(|| anyhow::anyhow!("memory limit is too large: {}", value))
        })
        .transpose()?
        .map(i64::try_from)
        .transpose()
        .context("memory limit exceeds i64 range")?;
    Ok(WorkerResourceLimits {
        cpu_limit_millicores,
        memory_limit_mb,
        nano_cpus,
        memory_bytes,
    })
}

pub(super) fn job_descriptor(payload: &WorkerJobPayload) -> (&'static str, &str, &str) {
    match &payload.action {
        WorkerAction::Parse(parse) => ("parse", parse.package_name.as_str(), "-"),
        WorkerAction::Build(build) => (
            "build",
            build.package_name.as_str(),
            build.mock_chroot.as_str(),
        ),
    }
}
