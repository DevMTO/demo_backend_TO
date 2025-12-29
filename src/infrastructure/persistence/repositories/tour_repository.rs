use async_trait::async_trait;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{debug, info, instrument};

use crate::application::ports::{TourRepositoryPort, PaginationOptions, PaginatedResult};
use crate::domain::{entities::Tour, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{TourModel, NewTourModel, UpdateTourModel},
    schema::tours,
};

pub struct PostgresTourRepository {
    pool: DatabasePool,
}

impl PostgresTourRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TourRepositoryPort for PostgresTourRepository {
    #[instrument(skip(self, tour))]
    async fn create(&self, tour: &Tour) -> Result<Tour, ApplicationError> {
        debug!("📝 Creando tour: {}", tour.nombre);
        let mut conn = self.pool.get_connection().await?;
        let new_tour: NewTourModel = tour.into();
        let result = diesel::insert_into(tours::table)
            .values(&new_tour)
            .get_result::<TourModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        info!("✅ Tour creado: {} (id: {})", result.nombre, result.id);
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<Tour>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = tours::table
            .filter(tours::id.eq(id))
            .first::<TourModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }
    
    async fn update(&self, tour: &Tour) -> Result<Tour, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let changes = UpdateTourModel {
            nombre: Some(&tour.nombre),
            lugar_inicio: Some(&tour.lugar_inicio),
            lugar_fin: Some(&tour.lugar_fin),
            hora_inicio: Some(tour.hora_inicio),
            hora_fin: Some(tour.hora_fin),
            detalles: Some(tour.detalles.clone()),
            itinerario: Some(tour.itinerario.clone()),
            precio_base: Some(tour.precio_base.clone()),
            duracion_dias: Some(tour.duracion_dias),
            max_personas: Some(tour.max_personas),
            media: Some(tour.media.clone()),
            is_active: Some(tour.is_active),
            updated_by: tour.updated_by,
        };
        let result = diesel::update(tours::table.filter(tours::id.eq(tour.id)))
            .set(&changes)
            .get_result::<TourModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(tours::table.filter(tours::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
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
        let total = self.count().await?;
        let limit = options.limit.unwrap_or(50);
        let offset = options.offset.unwrap_or(0);
        let data = self.list(limit, offset).await?;
        Ok(PaginatedResult::new(data, total, limit, offset))
    }
    
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(tours::table.filter(tours::id.eq(id)))
            .set((tours::is_active.eq(false), tours::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(tours::table.filter(tours::id.eq(id)))
            .set((tours::is_active.eq(true), tours::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
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
