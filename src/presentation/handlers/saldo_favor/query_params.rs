//! Query parameters para Saldo a Favor

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SaldoFavorQueryParams {
    pub id_entidad: Option<i32>,
    pub entidad: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size", alias = "per_page")]
    pub page_size: i64,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }
