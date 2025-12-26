//! # Presentation Layer
//! 
//! Capa de presentación con rutas HTTP y handlers.

pub mod routes;
pub mod handlers;
pub mod middleware;
pub mod extractors;

pub use routes::*;
