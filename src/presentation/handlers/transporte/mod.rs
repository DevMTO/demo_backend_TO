//! Módulo de handlers para Transporte
//! Dividido por método HTTP: get, post, put, patch, delete

pub mod get;
pub mod post;
pub mod put;
pub mod patch;
pub mod delete;
mod helpers;

pub use get::*;
pub use post::*;
pub use put::*;
pub use patch::*;
pub use delete::*;
