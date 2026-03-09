//! Query parameters para Contabilidad

use serde::Deserialize;

/// Parametros para listar pagos de files
#[derive(Debug, Deserialize)]
pub struct PagosFilesQueryParams {
    pub id_entidad: Option<i32>,
    pub estado: Option<String>,
    pub fecha_desde: Option<String>,
    pub fecha_hasta: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size", alias = "per_page")]
    pub page_size: i64,
}

/// Parametros para listar pagos a proveedores
#[derive(Debug, Deserialize)]
pub struct PagosProveedoresQueryParams {
    pub tipo_proveedor: Option<String>,
    pub estado: Option<String>,
    pub fecha_desde: Option<String>,
    pub fecha_hasta: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size", alias = "per_page")]
    pub page_size: i64,
}

pub fn default_page() -> i64 {
    1
}

pub fn default_page_size() -> i64 {
    20
}