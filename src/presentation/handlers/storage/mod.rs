//! Módulo de handlers para Storage
//! Gestión de archivos multimedia en Tigris (logos, banners, imágenes)

pub mod types;
pub mod helpers;
pub mod agencia;
pub mod cadena;
pub mod transporte;
pub mod tour;
pub mod proxy;

pub use agencia::*;
pub use cadena::*;
pub use transporte::*;
pub use tour::*;
pub use proxy::*;
