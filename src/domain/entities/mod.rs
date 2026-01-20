// Core Auth Entities
pub mod user;
pub mod session;

// System Entities
pub mod activity_log;
pub mod notification;

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
pub mod file_tour;
pub mod pago;

// Re-exports - Auth core
pub use user::{User, UserRole};
pub use session::UserSession;

// Re-exports - System entities
pub use activity_log::{
    ActivityLog, ActivityLogBuilder, NewActivityLog,
    ActionType, Action, LogStatus, EntityType,
};
pub use notification::{
    Notification, NotificationBuilder, NewNotification,
    NotificationUser, NewNotificationUser, NotificationWithReadStatus,
    NotificationType, NotificationCategory, NotificationPriority,
};

// Re-exports - Business entities
pub use persona::{Persona, TipoDocumento};
pub use agencia::Agencia;
pub use tour::Tour;
pub use transporte::Transporte;
pub use vehiculo::{Vehiculo, StatusVehiculo};
pub use conductor::{Conductor, StatusConductor};
pub use guia::{Guia, StatusGuia};
pub use restaurante::Restaurante;
pub use entrada::Entrada;
pub use file::File;
pub use file_tour::FileTour;
pub use pago::{Pago, TipoMovimiento};