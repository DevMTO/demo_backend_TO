use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{info, instrument};

use crate::application::ports::{VehiculoRepositoryPort, PaginationOptions, PaginatedResult};
use crate::application::dtos::VehiculoListItemDto;
use crate::domain::{entities::Vehiculo, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{VehiculoModel, NewVehiculoModel, UpdateVehiculoModel},
    schema::{vehiculos, transportes},
};

pub struct PostgresVehiculoRepository { pool: DatabasePool }

impl PostgresVehiculoRepository {
    pub fn new(pool: DatabasePool) -> Self { Self { pool } }
}

#[async_trait]
impl VehiculoRepositoryPort for PostgresVehiculoRepository {
    #[instrument(skip(self, vehiculo))]
    async fn create(&self, vehiculo: &Vehiculo) -> Result<Vehiculo, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new_v: NewVehiculoModel = vehiculo.into();
        let result = diesel::insert_into(vehiculos::table).values(&new_v)
            .get_result::<VehiculoModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        info!("Vehiculo creado: {} (id: {})", result.placa, result.id);
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<Vehiculo>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = vehiculos::table.filter(vehiculos::id.eq(id))
            .first::<VehiculoModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, vehiculo: &Vehiculo) -> Result<Vehiculo, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let status_str = vehiculo.status.to_string();
        let changes = UpdateVehiculoModel {
            nombre: Some(&vehiculo.nombre),
            modelo: Some(vehiculo.modelo.as_deref()),
            placa: Some(&vehiculo.placa),
            capacidad: Some(vehiculo.capacidad),
            status: Some(&status_str),
            is_active: Some(vehiculo.is_active),
            updated_by: vehiculo.updated_by,
        };
        let result = diesel::update(vehiculos::table.filter(vehiculos::id.eq(vehiculo.id)))
            .set(&changes).get_result::<VehiculoModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(vehiculos::table.filter(vehiculos::id.eq(id)))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(vehiculos::table.filter(vehiculos::id.eq(id)))
            .set((vehiculos::is_active.eq(false), vehiculos::updated_by.eq(Some(user_id))))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(vehiculos::table.filter(vehiculos::id.eq(id)))
            .set((vehiculos::is_active.eq(true), vehiculos::updated_by.eq(Some(user_id))))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    /// Eliminación permanente (hard delete)
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError> {
        self.delete(id).await
    }
    
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Vehiculo>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = vehiculos::table.order(vehiculos::placa.asc()).limit(limit).offset(offset)
            .load::<VehiculoModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn count(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        vehiculos::table.count().get_result::<i64>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Vehiculo>, ApplicationError> {
        let total = self.count().await?;
        let limit = options.limit.unwrap_or(50); let offset = options.offset.unwrap_or(0);
        let data = self.list(limit, offset).await?;
        Ok(PaginatedResult::new(data, total, limit, offset))
    }
    
    #[instrument(skip(self))]
    async fn list_with_details(&self, limit: i64, offset: i64) -> Result<(Vec<VehiculoListItemDto>, i64), ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let total: i64 = vehiculos::table
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let results: Vec<(VehiculoModel, Option<String>)> = vehiculos::table
            .left_join(transportes::table.on(vehiculos::id_transporte.eq(transportes::id)))
            .select((
                VehiculoModel::as_select(),
                transportes::nombre.nullable(),
            ))
            .order(vehiculos::placa.asc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let items: Vec<VehiculoListItemDto> = results
            .into_iter()
            .map(|(vehiculo, transporte_nombre)| {
                VehiculoListItemDto {
                    id: vehiculo.id,
                    id_transporte: vehiculo.id_transporte,
                    transporte_nombre,
                    nombre: vehiculo.nombre,
                    modelo: vehiculo.modelo,
                    placa: vehiculo.placa,
                    capacidad: vehiculo.capacidad,
                    status: vehiculo.status,
                    is_active: vehiculo.is_active,
                    created_at: vehiculo.created_at,
                    updated_at: vehiculo.updated_at,
                }
            })
            .collect();
        
        info!("Listados {} vehículos de {} total", items.len(), total);
        Ok((items, total))
    }
    
    async fn find_by_placa(&self, placa: &str) -> Result<Option<Vehiculo>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = vehiculos::table.filter(vehiculos::placa.eq(placa))
            .first::<VehiculoModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn exists_by_placa(&self, placa: &str) -> Result<bool, ApplicationError> {
        Ok(self.find_by_placa(placa).await?.is_some())
    }
    
    async fn find_by_transporte(&self, id_transporte: i32) -> Result<Vec<Vehiculo>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = vehiculos::table.filter(vehiculos::id_transporte.eq(id_transporte))
            .load::<VehiculoModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn find_by_transporte_with_details(&self, id_transporte: i32) -> Result<Vec<VehiculoListItemDto>, ApplicationError> {
        use crate::infrastructure::persistence::schema::{vehiculos, transportes};
        
        let mut conn = self.pool.get_connection().await?;
        let results = vehiculos::table
            .inner_join(transportes::table)
            .filter(vehiculos::id_transporte.eq(id_transporte))
            .select((
                vehiculos::id,
                vehiculos::id_transporte,
                vehiculos::nombre,
                vehiculos::modelo,
                vehiculos::placa,
                vehiculos::capacidad,
                vehiculos::status,
                vehiculos::is_active,
                vehiculos::created_at,
                vehiculos::updated_at,
                transportes::nombre,
            ))
            .load::<(i32, i32, String, Option<String>, String, i32, String, bool, chrono::NaiveDateTime, chrono::NaiveDateTime, String)>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        Ok(results
            .into_iter()
            .map(|(id, id_transporte, nombre, modelo, placa, capacidad, status, is_active, created_at, updated_at, transporte_nombre)| {
                VehiculoListItemDto {
                    id,
                    id_transporte,
                    transporte_nombre: Some(transporte_nombre),
                    nombre,
                    modelo,
                    placa,
                    capacidad,
                    status,
                    is_active,
                    created_at: DateTime::from_naive_utc_and_offset(created_at, Utc),
                    updated_at: DateTime::from_naive_utc_and_offset(updated_at, Utc),
                }
            })
            .collect())
    }
    
    async fn list_available(&self) -> Result<Vec<Vehiculo>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = vehiculos::table.filter(vehiculos::status.eq("disponible"))
            .load::<VehiculoModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn update_status(&self, id: i32, status: &str) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(vehiculos::table.filter(vehiculos::id.eq(id)))
            .set(vehiculos::status.eq(status))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    #[instrument(skip(self))]
    async fn list_by_transporte_paginated(&self, transporte_id: i32, limit: i64, offset: i64) -> Result<(Vec<VehiculoListItemDto>, i64), ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let total: i64 = vehiculos::table
            .filter(vehiculos::id_transporte.eq(transporte_id))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let results: Vec<(VehiculoModel, Option<String>)> = vehiculos::table
            .left_join(transportes::table.on(vehiculos::id_transporte.eq(transportes::id)))
            .filter(vehiculos::id_transporte.eq(transporte_id))
            .select((
                VehiculoModel::as_select(),
                transportes::nombre.nullable(),
            ))
            .order(vehiculos::placa.asc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let items: Vec<VehiculoListItemDto> = results
            .into_iter()
            .map(|(vehiculo, transporte_nombre)| {
                VehiculoListItemDto {
                    id: vehiculo.id,
                    id_transporte: vehiculo.id_transporte,
                    transporte_nombre,
                    nombre: vehiculo.nombre,
                    modelo: vehiculo.modelo,
                    placa: vehiculo.placa,
                    capacidad: vehiculo.capacidad,
                    status: vehiculo.status,
                    is_active: vehiculo.is_active,
                    created_at: vehiculo.created_at,
                    updated_at: vehiculo.updated_at,
                }
            })
            .collect();
        
        info!("Listados {} vehículos del transporte {} de {} total", items.len(), transporte_id, total);
        Ok((items, total))
    }
    
    async fn list_available_by_transporte(&self, transporte_id: i32) -> Result<Vec<Vehiculo>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = vehiculos::table
            .filter(vehiculos::id_transporte.eq(transporte_id))
            .filter(vehiculos::status.eq("disponible"))
            .filter(vehiculos::is_active.eq(true))
            .load::<VehiculoModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
}
