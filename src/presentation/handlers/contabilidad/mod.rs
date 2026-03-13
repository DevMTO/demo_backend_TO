//! Modulo de handlers para Contabilidad
//! Dividido por metodo HTTP: get, post

pub mod get;
pub mod post;
pub mod query_params;
pub mod liquidacion_detalle;

pub use get::*;
pub use post::*;
pub use liquidacion_detalle::*;