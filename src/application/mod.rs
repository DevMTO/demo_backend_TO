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

