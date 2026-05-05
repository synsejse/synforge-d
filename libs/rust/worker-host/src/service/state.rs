use synforge_core::{
    model::BuildTrigger,
    package::{PackageDefinition, SpecRevision},
};

#[derive(Debug, Clone)]
pub struct QueuedBuildRequest {
    pub package: PackageDefinition,
    pub mock_chroot: String,
    pub revision: SpecRevision,
    pub trigger: BuildTrigger,
    pub job_id: uuid::Uuid,
}
