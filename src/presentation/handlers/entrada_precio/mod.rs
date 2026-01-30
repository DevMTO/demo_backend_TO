//! Módulo de handlers para EntradaPrecios
//! Dividido por método HTTP: get, post, put, delete

pub mod query_params;
pub mod get;
pub mod post;
pub mod put;
pub mod delete;

pub use get::*;
pub use post::*;
pub use put::*;
pub use delete::*;
