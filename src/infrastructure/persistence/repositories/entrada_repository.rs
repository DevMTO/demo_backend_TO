use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{info, instrument};

use crate::application::ports::{EntradaRepositoryPort, PaginationOptions, PaginatedResult, CachePort, entity_names};
use crate::domain::{entities::Entrada, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{EntradaModel, NewEntradaModel, UpdateEntradaModel},
    schema::entradas,
};

pub struct PostgresEntradaRepository {
    pool: DatabasePool,
    cache: Arc<dyn CachePort>,
}

impl PostgresEntradaRepository {
    pub fn new(pool: DatabasePool, cache: Arc<dyn CachePort>) -> Self { Self { pool, cache } }
    
    fn list_cache_key(limit: i64, offset: i64, include_inactive: bool) -> String {
        format!("list:{}:{}:{}", limit, offset, include_inactive)
    }
    
    async fn invalidate_cache(&self) {
        self.cache.invalidate_entity(entity_names::ENTRADAS).await;
        info!("[CACHE INVALIDATED] entradas");
    }
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
        info!("Entrada creada: {} (id: {})", result.nombre, result.id);
        self.invalidate_cache().await;
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<Entrada>, ApplicationError> {
        let start = Instant::now();
        
        if let Some(cached) = self.cache.get_detail(entity_names::ENTRADAS, id).await {
            if let Ok(entrada) = serde_json::from_str::<Entrada>(&cached) {
                info!("[CACHE HIT] entrada #{} | {}ms", id, start.elapsed().as_millis());
                return Ok(Some(entrada));
            }
        }
        
        let mut conn = self.pool.get_connection().await?;
        let result = entradas::table.filter(entradas::id.eq(id))
            .first::<EntradaModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        if let Some(ref model) = result {
            let entrada: Entrada = model.clone().into();
            if let Ok(serialized) = serde_json::to_string(&entrada) {
                self.cache.set_detail(entity_names::ENTRADAS, id, serialized).await;
            }
            info!("[CACHE MISS → DB] entrada #{} | {}ms", id, start.elapsed().as_millis());
        }
        
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
            boleto_turistico: Some(entrada.boleto_turistico),
        };
        let result = diesel::update(entradas::table.filter(entradas::id.eq(entrada.id)))
            .set(&changes).get_result::<EntradaModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        self.invalidate_cache().await;
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(entradas::table.filter(entradas::id.eq(id)))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }
    
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(entradas::table.filter(entradas::id.eq(id)))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
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
        let start = Instant::now();
        let limit = options.limit.unwrap_or(50);
        let offset = options.offset.unwrap_or(0);
        let cache_key = Self::list_cache_key(limit, offset, false);
        
        if let Some(cached) = self.cache.get_list(entity_names::ENTRADAS, &cache_key).await {
            if let Ok(result) = serde_json::from_str::<PaginatedResult<Entrada>>(&cached) {
                info!("[CACHE HIT] entradas list '{}' | {}ms", cache_key, start.elapsed().as_millis());
                return Ok(result);
            }
        }
        
        let total = self.count().await?;
        let data = self.list(limit, offset).await?;
        let result = PaginatedResult::new(data, total, limit, offset);
        
        if let Ok(serialized) = serde_json::to_string(&result) {
            self.cache.set_list(entity_names::ENTRADAS, &cache_key, serialized).await;
        }
        
        info!("[CACHE MISS → DB] entradas list '{}' | {}ms", cache_key, start.elapsed().as_millis());
        Ok(result)
    }
    
    async fn list_all_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Entrada>, ApplicationError> {
        let start = Instant::now();
        let limit = options.limit.unwrap_or(50);
        let offset = options.offset.unwrap_or(0);
        let cache_key = Self::list_cache_key(limit, offset, true);
        
        if let Some(cached) = self.cache.get_list(entity_names::ENTRADAS, &cache_key).await {
            if let Ok(result) = serde_json::from_str::<PaginatedResult<Entrada>>(&cached) {
                info!("[CACHE HIT] entradas all list '{}' | {}ms", cache_key, start.elapsed().as_millis());
                return Ok(result);
            }
        }
        
        let total = self.count_all().await?;
        let data = self.list_all(limit, offset).await?;
        let result = PaginatedResult::new(data, total, limit, offset);
        
        if let Ok(serialized) = serde_json::to_string(&result) {
            self.cache.set_list(entity_names::ENTRADAS, &cache_key, serialized).await;
        }
        
        info!("[CACHE MISS → DB] entradas all list '{}' | {}ms", cache_key, start.elapsed().as_millis());
        Ok(result)
    }
    
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(entradas::table.filter(entradas::id.eq(id)))
            .set((entradas::is_active.eq(false), entradas::updated_by.eq(Some(user_id))))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }
    
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(entradas::table.filter(entradas::id.eq(id)))
            .set((entradas::is_active.eq(true), entradas::updated_by.eq(Some(user_id))))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }
}

