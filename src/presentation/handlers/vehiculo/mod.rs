//! Módulo de handlers para Vehiculo
//! Dividido por método HTTP: get, post, put, patch, delete

pub mod query_params;
pub mod get;
pub mod post;
pub mod put;
pub mod patch;
pub mod delete;

pub use get::*;
pub use post::*;
pub use put::*;
pub use patch::*;
pub use delete::*;
