// Auth DTOs
pub mod auth_dto;
pub mod user_dto;

// System DTOs
pub mod activity_log_dto;
pub mod notification_dto;

// Geo DTOs
pub mod geo_dto;

// Business Entity DTOs
pub mod persona_dto;
pub mod agencia_dto;
pub mod tour_dto;
pub mod transporte_dto;
pub mod vehiculo_dto;
pub mod conductor_dto;
pub mod guia_dto;
pub mod restaurante_dto;
pub mod entrada_dto;
pub mod entrada_precio_dto;
pub mod file_dto;
pub mod file_relations_dto;

// Contabilidad DTOs
pub mod contabilidad_dto;

// Re-exports - Auth
// auth_dto y user_dto se usan internamente

// Re-exports - System
pub use activity_log_dto::*;
pub use notification_dto::*;

// Re-exports - Geo
pub use geo_dto::*;

// Re-exports - Business entities
pub use persona_dto::*;
pub use agencia_dto::*;
pub use tour_dto::*;
pub use transporte_dto::*;
pub use vehiculo_dto::*;
pub use conductor_dto::*;
pub use guia_dto::*;
pub use restaurante_dto::*;
pub use entrada_dto::*;
pub use entrada_precio_dto::*;
pub use file_dto::*;
pub use file_relations_dto::*;
pub use user_dto::{
    UserListItemDto, 
    UserDetailDto, 
    CreateUserRequest, 
    UpdateUserRequest,
    AdminChangePasswordRequest,
};

// Re-exports - Contabilidad
pub use contabilidad_dto::*;

