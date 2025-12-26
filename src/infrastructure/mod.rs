//! # Infrastructure Layer
//! 
//! Capa de infraestructura con adaptadores para el exterior.
//! 
//! ## Arquitectura Hexagonal:
//! Los adaptadores implementan los puertos definidos en la capa de aplicación.

pub mod persistence;
pub mod security;
pub mod container;

pub use persistence::DatabasePool;
pub use security::{Argon2PasswordHasher, SecureSessionManager};
pub use container::DependencyContainer;