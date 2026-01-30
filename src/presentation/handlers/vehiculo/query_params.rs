//! Estructuras de query params para vehículos

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VehiculosQueryParams {
    pub include_deleted: Option<bool>,
    pub transporte_id: Option<i32>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}
