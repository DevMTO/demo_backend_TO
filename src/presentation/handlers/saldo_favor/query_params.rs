use serde::Deserialize;

/// Query params para listar cancelaciones
#[derive(Debug, Deserialize)]
pub struct CancelacionesQueryParams {
    pub id_agencia: Option<i32>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

/// Query params para listar movimientos
#[derive(Debug, Deserialize)]
pub struct MovimientosSaldoQueryParams {
    pub id_agencia: Option<i32>,
    pub tipo: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }
