//! Query parameters para Contabilidad

use serde::Deserialize;

/// Parámetros para listar movimientos
#[derive(Debug, Deserialize)]
pub struct MovimientosQueryParams {
    pub id_cuenta: Option<i32>,
    pub tipo: Option<String>,
    pub fecha_desde: Option<String>,
    pub fecha_hasta: Option<String>,
    pub referencia_tipo: Option<String>,
    pub referencia_id: Option<i32>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

/// Parámetros para listar pagos de files
#[derive(Debug, Deserialize)]
pub struct PagosFilesQueryParams {
    pub id_agencia: Option<i32>,
    pub estado: Option<String>,
    pub fecha_desde: Option<String>,
    pub fecha_hasta: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

/// Parámetros para listar pagos a proveedores
#[derive(Debug, Deserialize)]
pub struct PagosProveedoresQueryParams {
    pub tipo_proveedor: Option<String>,
    pub estado: Option<String>,
    pub fecha_desde: Option<String>,
    pub fecha_hasta: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

/// Parámetros para listar tarifas
#[derive(Debug, Deserialize)]
pub struct TarifasQueryParams {
    pub tipo_servicio: Option<String>,
    #[serde(default)]
    pub solo_activas: bool,
}

pub fn default_page() -> i64 {
    1
}

pub fn default_page_size() -> i64 {
    20
}
