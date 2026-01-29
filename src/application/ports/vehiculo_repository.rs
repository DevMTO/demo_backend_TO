use async_trait::async_trait;
use crate::domain::errors::ApplicationError;
use crate::domain::entities::Vehiculo;
use crate::application::dtos::VehiculoListItemDto;
use super::{PaginationOptions, PaginatedResult};

#[allow(dead_code)]
#[async_trait]
pub trait VehiculoRepositoryPort: Send + Sync {
    // CRUD básico
    async fn create(&self, vehiculo: &Vehiculo) -> Result<Vehiculo, ApplicationError>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Vehiculo>, ApplicationError>;
    async fn update(&self, vehiculo: &Vehiculo) -> Result<Vehiculo, ApplicationError>;
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError>;
    
    // Soft delete y restore
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError>;
    
    /// Eliminación permanente (hard delete) - Solo SuperAdmin
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Vehiculo>, ApplicationError>;
    async fn count(&self) -> Result<i64, ApplicationError>;
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Vehiculo>, ApplicationError>;
    
    // Listado con detalles del transporte
    async fn list_with_details(&self, limit: i64, offset: i64) -> Result<(Vec<VehiculoListItemDto>, i64), ApplicationError>;
    
    // Específicos de Vehiculo
    async fn find_by_placa(&self, placa: &str) -> Result<Option<Vehiculo>, ApplicationError>;
    async fn exists_by_placa(&self, placa: &str) -> Result<bool, ApplicationError>;
    async fn find_by_transporte(&self, id_transporte: i32) -> Result<Vec<Vehiculo>, ApplicationError>;
    async fn find_by_transporte_with_details(&self, id_transporte: i32) -> Result<Vec<VehiculoListItemDto>, ApplicationError>;
    async fn list_available(&self) -> Result<Vec<Vehiculo>, ApplicationError>;
    async fn update_status(&self, id: i32, status: &str) -> Result<bool, ApplicationError>;
}
