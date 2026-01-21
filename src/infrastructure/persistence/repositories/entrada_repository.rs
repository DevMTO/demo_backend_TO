use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{info, instrument};

use crate::application::ports::{EntradaRepositoryPort, PaginationOptions, PaginatedResult};
use crate::domain::{entities::Entrada, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{EntradaModel, NewEntradaModel, UpdateEntradaModel},
    schema::entradas,
};

pub struct PostgresEntradaRepository { pool: DatabasePool }

impl PostgresEntradaRepository {
    pub fn new(pool: DatabasePool) -> Self { Self { pool } }
}

#[async_trait]
impl EntradaRepositoryPort for PostgresEntradaRepository {
    #[instrument(skip(self, entrada))]
    async fn create(&self, entrada: &Entrada) -> Result<Entrada, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new_e: NewEntradaModel = entrada.into();
        let result = diesel::insert_into(entradas::table).values(&new_e)
            .get_result::<EntradaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        info!("✅ Entrada creada: {} (id: {})", result.nombre, result.id);
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<Entrada>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = entradas::table.filter(entradas::id.eq(id))
            .first::<EntradaModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, entrada: &Entrada) -> Result<Entrada, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let changes = UpdateEntradaModel {
            nombre: Some(&entrada.nombre),
            tours_asociados: Some(entrada.tours_asociados.clone()),
            descripcion: Some(entrada.descripcion.as_deref()),
            is_active: Some(entrada.is_active),
            updated_by: entrada.updated_by,
        };
        let result = diesel::update(entradas::table.filter(entradas::id.eq(entrada.id)))
            .set(&changes).get_result::<EntradaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(entradas::table.filter(entradas::id.eq(id)))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Entrada>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = entradas::table.filter(entradas::is_active.eq(true))
            .order(entradas::nombre.asc()).limit(limit).offset(offset)
            .load::<EntradaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<Entrada>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = entradas::table
            .order((entradas::is_active.desc(), entradas::nombre.asc()))
            .limit(limit).offset(offset)
            .load::<EntradaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn count(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        entradas::table.filter(entradas::is_active.eq(true)).count()
            .get_result::<i64>(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn count_all(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        entradas::table.count()
            .get_result::<i64>(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Entrada>, ApplicationError> {
        let total = self.count().await?;
        let limit = options.limit.unwrap_or(50); let offset = options.offset.unwrap_or(0);
        let data = self.list(limit, offset).await?;
        Ok(PaginatedResult::new(data, total, limit, offset))
    }
    
    async fn list_all_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Entrada>, ApplicationError> {
        let total = self.count_all().await?;
        let limit = options.limit.unwrap_or(50); let offset = options.offset.unwrap_or(0);
        let data = self.list_all(limit, offset).await?;
        Ok(PaginatedResult::new(data, total, limit, offset))
    }
    
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(entradas::table.filter(entradas::id.eq(id)))
            .set((entradas::is_active.eq(false), entradas::updated_by.eq(Some(user_id))))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(entradas::table.filter(entradas::id.eq(id)))
            .set((entradas::is_active.eq(true), entradas::updated_by.eq(Some(user_id))))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
}

