use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{debug, info, instrument};

use crate::application::ports::{TourRepositoryPort, PaginationOptions, PaginatedResult, CachePort, entity_names};
use crate::domain::{entities::Tour, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{TourModel, NewTourModel, UpdateTourModel},
    schema::tours,
};

pub struct PostgresTourRepository {
    pool: DatabasePool,
    cache: Arc<dyn CachePort>,
}

impl PostgresTourRepository {
    pub fn new(pool: DatabasePool, cache: Arc<dyn CachePort>) -> Self {
        Self { pool, cache }
    }
    
    fn list_cache_key(limit: i64, offset: i64, include_inactive: bool) -> String {
        format!("list:{}:{}:{}", limit, offset, include_inactive)
    }
    
    async fn invalidate_cache(&self) {
        self.cache.invalidate_entity(entity_names::TOURS).await;
        info!("[CACHE INVALIDATED] tours");
    }
}

#[async_trait]
impl TourRepositoryPort for PostgresTourRepository {
    #[instrument(skip(self, tour))]
    async fn create(&self, tour: &Tour) -> Result<Tour, ApplicationError> {
        debug!("Creando tour: {}", tour.nombre);
        let mut conn = self.pool.get_connection().await?;
        let new_tour: NewTourModel = tour.into();
        let result = diesel::insert_into(tours::table)
            .values(&new_tour)
            .get_result::<TourModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        info!("Tour creado: {} (id: {})", result.nombre, result.id);
        self.invalidate_cache().await;
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<Tour>, ApplicationError> {
        let start = Instant::now();
        
        // Check cache first
        if let Some(cached) = self.cache.get_detail(entity_names::TOURS, id).await {
            if let Ok(tour) = serde_json::from_str::<Tour>(&cached) {
                info!("[CACHE HIT] tour #{} | {}ms", id, start.elapsed().as_millis());
                return Ok(Some(tour));
            }
        }
        
        let mut conn = self.pool.get_connection().await?;
        let result = tours::table
            .filter(tours::id.eq(id))
            .first::<TourModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        if let Some(ref model) = result {
            let tour: Tour = model.clone().into();
            if let Ok(serialized) = serde_json::to_string(&tour) {
                self.cache.set_detail(entity_names::TOURS, id, serialized).await;
            }
            info!("[CACHE MISS → DB] tour #{} | {}ms", id, start.elapsed().as_millis());
        }
        
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, tour: &Tour) -> Result<Tour, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let changes = UpdateTourModel {
            nombre: Some(&tour.nombre),
            lugar_inicio: Some(tour.lugar_inicio.as_deref()),
            lugar_fin: Some(tour.lugar_fin.as_deref()),
            horarios: Some(tour.horarios.clone()),
            detalles: Some(tour.detalles.clone()),
            itinerario: Some(tour.itinerario.clone()),
            precio_base: Some(tour.precio_base.clone()),
            duracion_dias: Some(tour.duracion_dias),
            media: Some(tour.media.clone()),
            tipo_tour: Some(tour.tipo_tour.as_deref()),
            is_active: Some(tour.is_active),
            tiene_restaurante: Some(tour.tiene_restaurante),
            updated_by: tour.updated_by,
            geo_inicio: Some(tour.geo_inicio.clone()),
            geo_fin: Some(tour.geo_fin.clone()),
            geo_ruta: Some(tour.geo_ruta.clone()),
        };
        let result = diesel::update(tours::table.filter(tours::id.eq(tour.id)))
            .set(&changes)
            .get_result::<TourModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        self.invalidate_cache().await;
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(tours::table.filter(tours::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }
    
    /// Eliminación permanente (hard delete) - Borra de la base de datos
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(tours::table.filter(tours::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }
    
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Tour>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = tours::table
            .filter(tours::is_active.eq(true))
            .order(tours::nombre.asc())
            .limit(limit)
            .offset(offset)
            .load::<TourModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn count(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        tours::table
            .filter(tours::is_active.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Tour>, ApplicationError> {
        let start = Instant::now();
        let limit = options.limit.unwrap_or(50);
        let offset = options.offset.unwrap_or(0);
        let cache_key = Self::list_cache_key(limit, offset, false);
        
        // Check cache
        if let Some(cached) = self.cache.get_list(entity_names::TOURS, &cache_key).await {
            if let Ok(result) = serde_json::from_str::<PaginatedResult<Tour>>(&cached) {
                info!("[CACHE HIT] tours list '{}' | {}ms", cache_key, start.elapsed().as_millis());
                return Ok(result);
            }
        }
        
        let total = self.count().await?;
        let data = self.list(limit, offset).await?;
        let result = PaginatedResult::new(data, total, limit, offset);
        
        if let Ok(serialized) = serde_json::to_string(&result) {
            self.cache.set_list(entity_names::TOURS, &cache_key, serialized).await;
        }
        
        info!("[CACHE MISS → DB] tours list '{}' | {}ms", cache_key, start.elapsed().as_millis());
        Ok(result)
    }
    
    async fn list_all_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Tour>, ApplicationError> {
        let start = Instant::now();
        let limit = options.limit.unwrap_or(50);
        let offset = options.offset.unwrap_or(0);
        let cache_key = Self::list_cache_key(limit, offset, true);
        
        // Check cache
        if let Some(cached) = self.cache.get_list(entity_names::TOURS, &cache_key).await {
            if let Ok(result) = serde_json::from_str::<PaginatedResult<Tour>>(&cached) {
                info!("[CACHE HIT] tours all list '{}' | {}ms", cache_key, start.elapsed().as_millis());
                return Ok(result);
            }
        }
        
        let mut conn = self.pool.get_connection().await?;
        
        let total = tours::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let results = tours::table
            .order(tours::nombre.asc())
            .limit(limit)
            .offset(offset)
            .load::<TourModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let data = results.into_iter().map(Into::into).collect();
        let result = PaginatedResult::new(data, total, limit, offset);
        
        if let Ok(serialized) = serde_json::to_string(&result) {
            self.cache.set_list(entity_names::TOURS, &cache_key, serialized).await;
        }
        
        info!("[CACHE MISS → DB] tours all list '{}' | {}ms", cache_key, start.elapsed().as_millis());
        Ok(result)
    }
    
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(tours::table.filter(tours::id.eq(id)))
            .set((tours::is_active.eq(false), tours::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }
    
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(tours::table.filter(tours::id.eq(id)))
            .set((tours::is_active.eq(true), tours::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }
    
    async fn search(&self, query: &str) -> Result<Vec<Tour>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = tours::table
            .filter(tours::nombre.ilike(format!("%{}%", query)))
            .filter(tours::is_active.eq(true))
            .order(tours::nombre.asc())
            .load::<TourModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn find_by_nombre(&self, nombre: &str) -> Result<Vec<Tour>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = tours::table
            .filter(tours::nombre.ilike(format!("%{}%", nombre)))
            .filter(tours::is_active.eq(true))
            .load::<TourModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn find_by_precio_range(&self, min: BigDecimal, max: BigDecimal) -> Result<Vec<Tour>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = tours::table
            .filter(tours::precio_base.ge(min))
            .filter(tours::precio_base.le(max))
            .filter(tours::is_active.eq(true))
            .load::<TourModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn find_by_duracion(&self, dias: i32) -> Result<Vec<Tour>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = tours::table
            .filter(tours::duracion_dias.eq(dias))
            .filter(tours::is_active.eq(true))
            .load::<TourModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
}
