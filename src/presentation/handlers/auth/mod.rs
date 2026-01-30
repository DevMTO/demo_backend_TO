//! Handlers para autenticación
//! 
//! Organizados por método HTTP:
//! - get: Verificación de sesión, perfil, health check
//! - post: Login, logout
//! - put: Actualización de perfil

pub mod get;
pub mod post;
pub mod put;
mod helpers;

// Re-exports
pub use get::*;
pub use post::*;
pub use put::*;
