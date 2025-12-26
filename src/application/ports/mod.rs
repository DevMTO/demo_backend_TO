//! # Application Ports
//! 
//! Puertos de salida (interfaces) que el dominio necesita.
//! Los adaptadores implementan estos puertos.

pub mod user_repository;
pub mod session_repository;
pub mod password_hasher;
pub mod session_manager;

pub use user_repository::UserRepositoryPort;
pub use session_repository::SessionRepositoryPort;
pub use password_hasher::PasswordHasherPort;
pub use session_manager::{SessionManagerPort, SessionTokenData};

