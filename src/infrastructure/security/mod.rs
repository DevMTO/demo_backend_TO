//! # Security Infrastructure
//! 
//! Implementaciones de seguridad (hashing, sesiones seguras, etc.)
//! 
//! ## Características de Seguridad:
//! - **Sesiones con tokens opacos**: No JWT, tokens almacenados en BD
//! - **Cookies HttpOnly**: Protección contra XSS
//! - **HMAC-SHA256**: Para hash de tokens
//! - **Argon2id**: Para hash de contraseñas
//! - **Rotación de tokens**: Para mayor seguridad

pub mod argon2_hasher;
pub mod session_manager;

pub use argon2_hasher::*;
pub use session_manager::*;
