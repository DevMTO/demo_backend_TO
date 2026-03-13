// Auth DTOs
pub mod auth_dto;
pub mod user_dto;

// System DTOs
pub mod activity_log_dto;
pub mod notification_dto;

// Geo DTOs
pub mod geo_dto;

// Business Entity DTOs
pub mod agencia_dto;
pub mod conductor_dto;
pub mod entrada_dto;
pub mod entrada_precio_dto;
pub mod file_dto;
pub mod file_relations_dto;
pub mod guia_dto;
pub mod persona_dto;
pub mod restaurante_dto;
pub mod tour_dto;
pub mod transporte_dto;
pub mod vehiculo_dto;

// Hotel DTOs
pub mod cadena_hotelera_dto;
pub mod hotel_dto;

// Tarifa DTOs
pub mod tarifa_dto;

// Contabilidad DTOs
pub mod contabilidad_dto;

// Chat DTOs
pub mod chat_dto;

// Common DTOs
pub mod common;

// Re-exports - Auth
// auth_dto y user_dto se usan internamente

// Re-exports - System
pub use activity_log_dto::*;
pub use notification_dto::*;

// Re-exports - Common
pub use common::*;

// Re-exports - Geo
pub use geo_dto::*;

// Re-exports - Business entities
pub use agencia_dto::*;
pub use conductor_dto::*;
pub use entrada_dto::*;
pub use entrada_precio_dto::*;
pub use file_dto::*;
pub use file_relations_dto::*;
pub use guia_dto::*;
pub use persona_dto::*;
pub use restaurante_dto::*;
pub use tour_dto::*;
pub use transporte_dto::*;
pub use user_dto::{
    AdminChangePasswordRequest, CreateUserRequest, UpdateUserRequest, UserDetailDto,
    UserListItemDto,
};
pub use vehiculo_dto::*;

// Re-exports - Hotel
pub use cadena_hotelera_dto::*;
pub use hotel_dto::*;

// Re-exports - Tarifa
pub use tarifa_dto::*;

// Re-exports - Contabilidad
pub use contabilidad_dto::*;

// Re-exports - Chat
pub use chat_dto::*;
