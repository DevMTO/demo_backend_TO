//! # Application Layer
//! 
//! Capa de aplicación con casos de uso y DTOs.
//! 
//! ## Arquitectura Hexagonal:
//! Esta capa orquesta los casos de uso del sistema, utilizando
//! los puertos para comunicarse con el exterior.

pub mod ports;
pub mod use_cases;
pub mod dtos;

// Re-exports específicos
pub use dtos::auth_dto::{
    LoginRequest, RegisterRequest, AuthResponse, AuthUserInfo,
    ChangePasswordRequest, LogoutRequest, SuccessResponse,
};
pub use dtos::user_dto::{
    UserDetailDto, CreateUserRequest, UpdateUserRequest,
    UserListResponse, PaginationParams,
};
pub use ports::password_hasher::PasswordHasherPort;
pub use use_cases::auth::{LoginUseCase, LogoutUseCase, VerifySessionUseCase};

