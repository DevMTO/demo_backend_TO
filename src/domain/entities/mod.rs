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

// Re-exports - Auth
pub use user::{User, UserInfo, UserRole, UserStatus};
pub use session::UserSession;

// Re-exports - Tour Operator
pub use persona::Persona;
pub use agencia::Agencia;
pub use tour::Tour;
pub use transporte::Transporte;
pub use vehiculo::Vehiculo;
pub use conductor::Conductor;
pub use guia::Guia;
pub use restaurante::Restaurante;
pub use entrada::Entrada;
pub use file::File;
pub use pago::Pago;