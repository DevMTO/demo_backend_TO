use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{info, instrument};

use crate::application::ports::{TransporteRepositoryPort, PaginationOptions, PaginatedResult};
use crate::application::dtos::TransporteListItemDto;
use crate::domain::{entities::Transporte, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{TransporteModel, NewTransporteModel, UpdateTransporteModel},
    schema::{transportes, personas},
};

pub struct PostgresTransporteRepository { pool: DatabasePool }

impl PostgresTransporteRepository {
    pub fn new(pool: DatabasePool) -> Self { Self { pool } }
}

#[async_trait]
impl TransporteRepositoryPort for PostgresTransporteRepository {
    #[instrument(skip(self, transporte))]
    async fn create(&self, transporte: &Transporte) -> Result<Transporte, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new_t: NewTransporteModel = transporte.into();
        let result = diesel::insert_into(transportes::table).values(&new_t)
            .get_result::<TransporteModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        info!("✅ Transporte creado: {} (id: {})", result.nombre, result.id);
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<Transporte>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = transportes::table.filter(transportes::id.eq(id))
            .first::<TransporteModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, transporte: &Transporte) -> Result<Transporte, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let changes = UpdateTransporteModel {
            nombre: Some(&transporte.nombre), ruc: Some(&transporte.ruc),
            telefono: Some(transporte.telefono.as_deref()), correo: Some(transporte.correo.as_deref()),
            direccion: Some(transporte.direccion.as_deref()), encargado: Some(transporte.encargado),
            media: Some(transporte.media.clone()),
            paleta_colores: Some(transporte.paleta_colores.clone()),
            is_active: Some(transporte.is_active), updated_by: transporte.updated_by,
        };
        let result = diesel::update(transportes::table.filter(transportes::id.eq(transporte.id)))
            .set(&changes).get_result::<TransporteModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(transportes::table.filter(transportes::id.eq(id)))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Transporte>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = transportes::table.filter(transportes::is_active.eq(true))
            .order(transportes::nombre.asc()).limit(limit).offset(offset)
            .load::<TransporteModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn count(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        transportes::table.filter(transportes::is_active.eq(true)).count()
            .get_result::<i64>(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Transporte>, ApplicationError> {
        let total = self.count().await?;
        let limit = options.limit.unwrap_or(50); let offset = options.offset.unwrap_or(0);
        let data = self.list(limit, offset).await?;
        Ok(PaginatedResult::new(data, total, limit, offset))
    }
    
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(transportes::table.filter(transportes::id.eq(id)))
            .set((transportes::is_active.eq(false), transportes::updated_by.eq(Some(user_id))))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(transportes::table.filter(transportes::id.eq(id)))
            .set((transportes::is_active.eq(true), transportes::updated_by.eq(Some(user_id))))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn find_by_ruc(&self, ruc: &str) -> Result<Option<Transporte>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = transportes::table.filter(transportes::ruc.eq(ruc))
            .first::<TransporteModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn exists_by_ruc(&self, ruc: &str) -> Result<bool, ApplicationError> {
        Ok(self.find_by_ruc(ruc).await?.is_some())
    }
    
    async fn find_with_available_vehicles(&self) -> Result<Vec<Transporte>, ApplicationError> {
        // Simplificado - retorna todos los transportes activos
        self.list(100, 0).await
    }
    
    async fn list_with_encargado(&self, limit: i64, offset: i64) -> Result<(Vec<TransporteListItemDto>, i64), ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        // Contar total
        let total: i64 = transportes::table
            .filter(transportes::is_active.eq(true))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        // Query con LEFT JOIN a personas para obtener nombre del encargado
        let results: Vec<(TransporteModel, Option<(String, String)>)> = transportes::table
            .left_join(personas::table.on(transportes::encargado.eq(personas::id.nullable())))
            .select((
                TransporteModel::as_select(),
                (personas::nombre, personas::apellidos).nullable()
            ))
            .filter(transportes::is_active.eq(true))
            .order(transportes::nombre.asc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let items: Vec<TransporteListItemDto> = results.into_iter().map(|(transporte, encargado_data)| {
            let encargado_nombre = encargado_data.map(|(nombre, apellidos)| format!("{} {}", nombre, apellidos));
            TransporteListItemDto {
                id: transporte.id,
                nombre: transporte.nombre,
                ruc: transporte.ruc,
                telefono: transporte.telefono,
                correo: transporte.correo,
                direccion: transporte.direccion,
                encargado: transporte.encargado,
                encargado_nombre,
                media: transporte.media,
                paleta_colores: transporte.paleta_colores,
                is_active: transporte.is_active,
                created_at: transporte.created_at,
                updated_at: transporte.updated_at,
            }
        }).collect();
        
        Ok((items, total))
    }
    
    async fn find_by_encargado(&self, persona_id: i32) -> Result<Option<Transporte>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = transportes::table
            .filter(transportes::encargado.eq(Some(persona_id)))
            .filter(transportes::is_active.eq(true))
            .first::<TransporteModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
}
