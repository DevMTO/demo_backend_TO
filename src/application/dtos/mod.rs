//! # Application DTOs
//!
//! Data Transfer Objects para la capa de aplicación.

pub mod auth_dto;
pub mod user_dto;

// Re-exportar DTOs de autenticación para uso interno
pub use auth_dto::{
    AuthResponse,
    LoginRequest,
    LogoutRequest,
    RegisterRequest,
    SuccessResponse,
    AuthUserInfo,
    ChangePasswordRequest,
};

pub use user_dto::{
    UserDetailDto,
    CreateUserRequest,
    UpdateUserRequest,
    UserListResponse,
    PaginationParams,
};
pub mod ts;

