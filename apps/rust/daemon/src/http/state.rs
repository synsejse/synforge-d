use std::sync::Arc;

use crate::SynforgeService;

#[derive(Clone)]
pub(crate) struct AppState {
    pub service: Arc<SynforgeService>,
}
