//! Módulo de handlers para Contabilidad
//! Dividido por método HTTP: get, post, put, delete

pub mod get;
pub mod post;
pub mod put;
pub mod delete;
pub mod query_params;

pub use get::*;
pub use post::*;
pub use put::*;
pub use delete::*;
