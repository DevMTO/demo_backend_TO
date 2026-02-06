//! Query parameters para Entrada

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct EntradaListParams {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    #[serde(default)]
    pub include_inactive: bool,
}

impl EntradaListParams {
    pub fn to_options(&self) -> crate::application::ports::PaginationOptions {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(50).clamp(1, 500);
        let offset = (page - 1) * page_size;
        crate::application::ports::PaginationOptions { limit: Some(page_size), offset: Some(offset) }
    }
}
