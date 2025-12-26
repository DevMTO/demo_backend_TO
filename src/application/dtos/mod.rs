//! # Application DTOs
//!
//! Data Transfer Objects para la capa de aplicación.
//!
//! ## TypeScript Types (ts-rs)
//!
//! Los tipos TypeScript están organizados en módulos separados dentro de `ts/`.
//! Para generar los tipos, ejecutar:
//! ```bash
//! cargo test export_ts_types -- --nocapture
//! ```
//! Los archivos se generan en `../../frontend/src/domain/contracts/`

pub mod auth_dto;
pub mod user_dto;

// Re-exportar DTOs de autenticación para uso interno
pub use auth_dto::{
    AuthResponse,
    LoginRequest,
    LogoutRequest,
    RegisterRequest,
    SuccessResponse,
    UserInfo,
};

// Módulo ts/ contiene tipos para generación TypeScript (ts-rs)
// NO lo re-exportamos aquí porque son tipos solo para exportar a frontend
// Los tests de exportación están en ts/mod.rs
// Ejecutar con: cargo test export_ts_types -- --nocapture
pub mod ts;

