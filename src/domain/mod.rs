pub mod entities;
pub mod value_objects;
pub mod errors;
pub mod services;

// Re-exports para facilitar el uso
pub use entities::{
    User, UserInfo, UserRole, UserStatus, UserSession,
    Persona, Agencia, Tour, Transporte, Vehiculo,
    Conductor, Guia, Restaurante, Entrada, File, Pago,
};
pub use errors::DomainError;
