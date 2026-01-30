//! Módulo de handlers para Notification
//! Dividido por método HTTP: get, post, patch, delete, sse

pub mod get;
pub mod post;
pub mod patch;
pub mod delete;
pub mod sse;
pub mod query_params;

pub use get::*;
pub use post::*;
pub use patch::*;
pub use delete::*;
pub use sse::*;
