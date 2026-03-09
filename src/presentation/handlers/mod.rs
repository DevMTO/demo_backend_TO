//! Módulo de handlers - Organizado por entidad con archivos separados por método HTTP

pub mod common;

// Handlers organizados en carpetas por entidad
pub mod activity_log;
pub mod agencia;
pub mod auth;
pub mod conductor;
pub mod contabilidad;
pub mod entrada;
pub mod entrada_precio;
pub mod file;
pub mod file_relations;
pub mod guia;
pub mod my_files;
pub mod notification;
pub mod persona;
pub mod relation_status;
pub mod restaurante;
pub mod storage;
pub mod tour;
pub mod transporte;
pub mod user;
pub mod vehiculo;
pub mod mis_pagos;
pub mod saldo_favor;
pub mod cadena_hotelera;
pub mod hotel;

// Re-exports para compatibilidad con rutas existentes
pub use auth::{login_handler, logout_handler, verify_session_handler, health_check, get_profile_handler, update_profile_handler};

