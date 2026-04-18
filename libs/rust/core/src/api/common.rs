use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PageInfo {
    pub limit: usize,
    pub offset: usize,
    pub returned: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, IntoParams, ToSchema)]
pub struct PaginationQuery {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
}

const DEFAULT_PAGE_SIZE: usize = 50;
const MAX_PAGE_SIZE: usize = 200;

pub fn normalize_pagination(limit: Option<usize>, offset: Option<usize>) -> (usize, usize) {
    (
        limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE),
        offset.unwrap_or(0),
    )
}

pub fn build_page_info(limit: usize, offset: usize, total: u64, returned: usize) -> PageInfo {
    PageInfo {
        limit,
        offset,
        returned,
        total: Some(total),
        has_more: (offset as u64) + (returned as u64) < total,
    }
}
