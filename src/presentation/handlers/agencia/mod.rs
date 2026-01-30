//! Handlers para el módulo de Agencias
//! 
//! Organizados por método HTTP:
//! - get: Consultas (list, get by id, get by ruc, get mi agencia)
//! - post: Creación
//! - put: Actualización completa
//! - patch: Actualización parcial (interfaz, restaurar)
//! - delete: Eliminación (soft y hard)

pub mod get;
pub mod post;
pub mod put;
pub mod patch;
pub mod delete;

// Re-exports para facilitar el uso
pub use get::*;
pub use post::*;
pub use put::*;
pub use patch::*;
pub use delete::*;
