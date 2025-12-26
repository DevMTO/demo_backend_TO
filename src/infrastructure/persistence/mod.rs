//! # Persistence Infrastructure
//! 
//! Adaptadores de persistencia (base de datos).

pub mod database;
pub mod schema;
pub mod models;
pub mod repositories;

pub use database::*;
pub use schema::*;
