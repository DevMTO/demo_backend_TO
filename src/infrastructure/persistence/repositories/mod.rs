// Auth repositories
pub mod user_repository;
pub mod session_repository;

// System repositories
pub mod activity_log_repository;
pub mod notification_repository;

// Business entity repositories
pub mod persona_repository;
pub mod agencia_repository;
pub mod tour_repository;
pub mod transporte_repository;
pub mod vehiculo_repository;
pub mod conductor_repository;
pub mod guia_repository;
pub mod restaurante_repository;
pub mod entrada_repository;
pub mod entrada_precio_repository;
pub mod file_repository;
pub mod file_relations_repository;
pub mod pago_repository;

// Contabilidad repositories
pub mod contabilidad_repository;

// Re-exports - Auth
pub use user_repository::*;
pub use session_repository::*;

// Re-exports - System
pub use activity_log_repository::*;
pub use notification_repository::*;

// Re-exports - Business entities
pub use persona_repository::*;
pub use agencia_repository::*;
pub use tour_repository::*;
pub use transporte_repository::*;
pub use vehiculo_repository::*;
pub use conductor_repository::*;
pub use guia_repository::*;
pub use restaurante_repository::*;
pub use entrada_repository::*;
pub use entrada_precio_repository::*;
pub use file_repository::*;
pub use file_relations_repository::*;
pub use pago_repository::*;

// Re-exports - Contabilidad
pub use contabilidad_repository::*;

// Saldo a favor
pub mod saldo_favor_repository;
pub use saldo_favor_repository::*;

