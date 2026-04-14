use synforge_core::api::PageInfo;

const DEFAULT_PAGE_SIZE: usize = 50;
const MAX_PAGE_SIZE: usize = 200;

pub(crate) fn normalize_pagination(limit: Option<usize>, offset: Option<usize>) -> (usize, usize) {
    (
        limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE),
        offset.unwrap_or(0),
    )
}

pub(crate) fn build_page_info(
    limit: usize,
    offset: usize,
    total: u64,
    returned: usize,
) -> PageInfo {
    PageInfo {
        limit,
        offset,
        returned,
        total: Some(total),
        has_more: (offset as u64) + (returned as u64) < total,
    }
}
