//! Query params para EntradaPrecio

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CalcularPrecioQuery {
    pub edad: i32,
    pub tipo_turista: String, // "nacional" o "extranjero"
}
