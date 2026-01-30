//! Módulo de handlers para Activity Log
//! Solo GET handlers (logs son inmutables)

pub mod get;
pub mod query_params;

pub use get::*;
