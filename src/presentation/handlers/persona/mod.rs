//! Módulo de handlers para Persona
//! Dividido por método HTTP: get, post, put, delete

pub mod get;
pub mod post;
pub mod put;
pub mod delete;

pub use get::*;
pub use post::*;
pub use put::*;
pub use delete::*;
