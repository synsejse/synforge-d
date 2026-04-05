mod job;
mod package;
mod repo;
mod user;

pub use job::JobStore;
pub use package::PackageStore;
pub use repo::RepoStore;
pub use user::UserStore;

use async_trait::async_trait;

/// Combined database trait that aggregates all domain-specific stores.
/// Implementations provide access to packages, jobs, repository files, and users.
#[async_trait]
pub trait Store: PackageStore + JobStore + RepoStore + UserStore + Send + Sync {}

// Blanket implementation: any type implementing all four traits automatically implements Store
impl<T> Store for T where T: PackageStore + JobStore + RepoStore + UserStore + Send + Sync {}
