//! # Domain Entities
//! 
//! Entidades de dominio del sistema Tour Operator según diagrama de base de datos.

// Core Auth Entities
pub mod user;
pub mod session;

// Tour Operator Business Entities
pub mod persona;
pub mod agencia;
pub mod tour;
pub mod transporte;
pub mod vehiculo;
pub mod conductor;
pub mod guia;
pub mod restaurante;
pub mod entrada;
pub mod file;
pub mod pago;

// Re-exports - Solo los tipos usados activamente
// Auth core
pub use user::{User, UserRole, UserStatus};
pub use session::UserSession;