//! Modulo de handlers para Contabilidad
//! Dividido por metodo HTTP: get, post

pub mod get;
pub mod post;
pub mod query_params;

pub use get::*;
pub use post::*;