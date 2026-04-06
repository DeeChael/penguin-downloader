use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Pagination {
    pub per_page: i32,
    pub page: i32,
}

impl Pagination {
    pub fn new(per_page: i32, page: i32) -> Self {
        Self {
            per_page: per_page.max(1),
            page: page.max(1),
        }
    }

    pub fn offset(&self) -> i32 {
        (self.page - 1) * self.per_page
    }
}

impl Pagination {
    pub fn default_search() -> Self {
        Self {
            per_page: 10,
            page: 1,
        }
    }

    pub fn default_list() -> Self {
        Self {
            per_page: 100,
            page: 1,
        }
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::default_search()
    }
}
