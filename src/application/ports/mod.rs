// Core ports
pub mod generic_repository;
pub mod user_repository;
pub mod session_repository;
pub mod password_hasher;
pub mod session_manager;

// System ports
pub mod activity_log_repository;
pub mod notification_repository;
pub mod notification_service;

// Business entity ports
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
pub mod pago_repository;

// File relations ports
pub mod file_relations_repository;

// Contabilidad ports
pub mod contabilidad_repository;

// Re-exports core
pub use generic_repository::{PaginationOptions, PaginatedResult};
pub use user_repository::UserRepositoryPort;
pub use session_repository::SessionRepositoryPort;
pub use password_hasher::PasswordHasherPort;
pub use session_manager::{SessionManagerPort, SessionTokenData};

// Re-exports system
pub use activity_log_repository::{ActivityLogRepositoryPort, ActivityLogFilters, CountByType};
pub use notification_repository::{NotificationRepositoryPort, NotificationFilters, PriorityCount, CleanupResult};
pub use notification_service::NotificationServicePort;

// Re-exports business entities
pub use persona_repository::PersonaRepositoryPort;
pub use agencia_repository::AgenciaRepositoryPort;
pub use tour_repository::TourRepositoryPort;
pub use transporte_repository::TransporteRepositoryPort;
pub use vehiculo_repository::VehiculoRepositoryPort;
pub use conductor_repository::ConductorRepositoryPort;
pub use guia_repository::GuiaRepositoryPort;
pub use restaurante_repository::RestauranteRepositoryPort;
pub use entrada_repository::EntradaRepositoryPort;
pub use entrada_precio_repository::EntradaPrecioRepositoryPort;
pub use file_repository::FileRepositoryPort;
pub use pago_repository::PagoRepositoryPort;

// Re-exports file relations
pub use file_relations_repository::{
    FileEntradaRepositoryPort, FileGuiaRepositoryPort, FilePasajeroRepositoryPort,
    FileRestauranteRepositoryPort, FileVehiculoRepositoryPort, FileTourRepositoryPort,
    FileTourInputData,
};

// Re-exports contabilidad
pub use contabilidad_repository::{
    CuentaRepositoryPort, MovimientoRepositoryPort, PagoFileRepositoryPort,
    PagoProveedorRepositoryPort, TarifaServicioRepositoryPort,
};
