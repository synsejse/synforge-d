//! Worker infrastructure adapters for the rewrite target architecture.

mod build_runner;
mod container;
mod hardware;
mod job_lifecycle;
mod job_usage;
mod log_broadcaster;
mod mock_chroots;
mod resources;
mod service;
mod sessions;
mod worker_socket;

use synforge_core::{
    model::BuildTrigger,
    package::{PackageDefinition, SpecRevision},
};
use tokio::sync::mpsc;
use uuid::Uuid;

pub mod launcher;

pub use build_runner::BuildRunner;
pub use hardware::HostHardwareService;
pub use job_lifecycle::JobLifecycle;
pub use job_usage::JobUsageService;
pub use launcher::{DockerWorkerLauncher, WorkerJobTracker};
pub use log_broadcaster::{LogBroadcaster, LogEvent};
pub use mock_chroots::MockChrootService;
pub use service::{
    ActiveTargetBuildReader, BuildJobReader, BuildJobWriter, BuildQueue, BuildService,
    ExistingSourceSyncer, LastSuccessfulRevisionReader, PackageDefinitionCatalog,
    PackageDefinitionReader, QueuedBuildRequest, RetryBuildCleaner, RetryJobResetter,
    TargetBuildBackoffReader, TrackedSourceInspector,
};
pub use sessions::{ActiveWorkerSession, WorkerSession, WorkerSessionBroker};
pub use worker_socket::start_worker_listener;

#[derive(Clone)]
pub struct QueuedBuild {
    pub package: PackageDefinition,
    pub mock_chroot: String,
    pub revision: SpecRevision,
    pub trigger: BuildTrigger,
    pub job_id: Uuid,
}

#[derive(Clone)]
pub struct WorkerBuildQueue {
    queue_tx: mpsc::Sender<QueuedBuild>,
}

impl WorkerBuildQueue {
    pub fn new(queue_tx: mpsc::Sender<QueuedBuild>) -> Self {
        Self { queue_tx }
    }
}
