use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{info, instrument};

use crate::application::dtos::RestauranteListItemDto;
use crate::application::ports::{RestauranteRepositoryPort, PaginationOptions, PaginatedResult, CachePort, entity_names};
use crate::domain::{entities::Restaurante, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{RestauranteModel, NewRestauranteModel, UpdateRestauranteModel},
    schema::{restaurantes, personas},
};

pub struct PostgresRestauranteRepository {
    pool: DatabasePool,
    cache: Arc<dyn CachePort>,
}

impl PostgresRestauranteRepository {
    pub fn new(pool: DatabasePool, cache: Arc<dyn CachePort>) -> Self {
        Self { pool, cache }
    }

    fn list_cache_key(limit: i64, offset: i64) -> String {
        format!("list:{}:{}", limit, offset)
    }

    async fn invalidate_cache(&self) {
        self.cache.invalidate_entity(entity_names::RESTAURANTES).await;
        info!("[CACHE INVALIDATED] restaurantes");
    }
}

#[async_trait]
impl RestauranteRepositoryPort for PostgresRestauranteRepository {
    #[instrument(skip(self, restaurante))]
    async fn create(&self, restaurante: &Restaurante) -> Result<Restaurante, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let new_r: NewRestauranteModel = restaurante.into();
        let result = diesel::insert_into(restaurantes::table).values(&new_r)
            .get_result::<RestauranteModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        info!("Restaurante creado: {} (id: {})", result.nombre, result.id);
        self.invalidate_cache().await;
        Ok(result.into())
    }
    
    async fn find_by_id(&self, id: i32) -> Result<Option<Restaurante>, ApplicationError> {
        let start = Instant::now();
        
        // Check cache
        if let Some(cached) = self.cache.get_detail(entity_names::RESTAURANTES, id).await {
            if let Ok(restaurante) = serde_json::from_str::<Restaurante>(&cached) {
                info!("[CACHE HIT] restaurante #{} | {}ms", id, start.elapsed().as_millis());
                return Ok(Some(restaurante));
            }
        }
        
        let mut conn = self.pool.get_connection().await?;
        let result = restaurantes::table.filter(restaurantes::id.eq(id))
            .first::<RestauranteModel>(&mut conn).await.optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        if let Some(ref model) = result {
            let restaurante: Restaurante = model.clone().into();
            if let Ok(serialized) = serde_json::to_string(&restaurante) {
                self.cache.set_detail(entity_names::RESTAURANTES, id, serialized).await;
            }
            info!("[CACHE MISS \u{2192} DB] restaurante #{} | {}ms", id, start.elapsed().as_millis());
            return Ok(Some(restaurante));
        }
        
        Ok(None)
    }
    
    async fn update(&self, restaurante: &Restaurante) -> Result<Restaurante, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let changes = UpdateRestauranteModel {
            nombre: Some(&restaurante.nombre),
            direccion: Some(&restaurante.direccion),
            telefono: Some(restaurante.telefono.as_deref()),
            correo: Some(restaurante.correo.as_deref()),
            tipo_atencion: Some(restaurante.tipo_atencion.clone()),
            precio_promedio: Some(restaurante.precio_promedio.clone()),
            capacidad: Some(restaurante.capacidad),
            horario: Some(restaurante.horario.clone()),
            is_active: Some(restaurante.is_active),
            updated_by: restaurante.updated_by,
            encargado: Some(restaurante.encargado),
        };
        let result = diesel::update(restaurantes::table.filter(restaurantes::id.eq(restaurante.id)))
            .set(&changes).get_result::<RestauranteModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.into())
    }
    
    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(restaurantes::table.filter(restaurantes::id.eq(id)))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    /// Eliminación permanente (hard delete) - Borra de la base de datos
    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(restaurantes::table.filter(restaurantes::id.eq(id)))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(affected > 0)
    }
    
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Restaurante>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = restaurantes::table.filter(restaurantes::is_active.eq(true))
            .order(restaurantes::nombre.asc()).limit(limit).offset(offset)
            .load::<RestauranteModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn count(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        restaurantes::table.filter(restaurantes::is_active.eq(true)).count()
            .get_result::<i64>(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    async fn list_paginated(&self, options: PaginationOptions) -> Result<PaginatedResult<Restaurante>, ApplicationError> {
        let total = self.count().await?;
        let limit = options.limit.unwrap_or(50); let offset = options.offset.unwrap_or(0);
        let data = self.list(limit, offset).await?;
        Ok(PaginatedResult::new(data, total, limit, offset))
    }
    
    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(restaurantes::table.filter(restaurantes::id.eq(id)))
            .set((restaurantes::is_active.eq(false), restaurantes::updated_by.eq(Some(user_id))))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }
    
    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(restaurantes::table.filter(restaurantes::id.eq(id)))
            .set((restaurantes::is_active.eq(true), restaurantes::updated_by.eq(Some(user_id))))
            .execute(&mut conn).await.map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }
    
    async fn find_by_tipo_atencion(&self, tipo: &str) -> Result<Vec<Restaurante>, ApplicationError> {
        // tipo_atencion es JSONB, no se puede filtrar directamente con eq()
        // Filtramos en memoria ya que JSONB contains no está disponible fácilmente
        let mut conn = self.pool.get_connection().await?;
        let all_results = restaurantes::table
            .filter(restaurantes::is_active.eq(true))
            .load::<RestauranteModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        // Filtrar en memoria por tipo_atencion
        let filtered: Vec<_> = all_results.into_iter()
            .filter(|r| {
                r.tipo_atencion.as_ref()
                    .and_then(|json| json.as_array())
                    .map(|arr| arr.iter().any(|v| v.as_str() == Some(tipo)))
                    .unwrap_or(false)
            })
            .map(Into::into)
            .collect();
        Ok(filtered)
    }
    
    async fn find_by_min_capacity(&self, min_capacity: i32) -> Result<Vec<Restaurante>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = restaurantes::table.filter(restaurantes::capacidad.ge(min_capacity))
            .filter(restaurantes::is_active.eq(true))
            .load::<RestauranteModel>(&mut conn).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }
    
    async fn list_with_encargado(&self, limit: i64, offset: i64) -> Result<(Vec<RestauranteListItemDto>, i64), ApplicationError> {
        let start = Instant::now();
        let cache_key = Self::list_cache_key(limit, offset);
        
        // Check cache
        if let Some(cached) = self.cache.get_list(entity_names::RESTAURANTES, &cache_key).await {
            if let Ok(response) = serde_json::from_str::<(Vec<RestauranteListItemDto>, i64)>(&cached) {
                info!("[CACHE HIT] restaurantes list '{}' | {}ms", cache_key, start.elapsed().as_millis());
                return Ok(response);
            }
        }
        
        let mut conn = self.pool.get_connection().await?;
        
        // Count total (todos, activos e inactivos)
        let total = restaurantes::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        // LEFT JOIN with personas to get encargado_nombre (todos, activos e inactivos)
        let results: Vec<(RestauranteModel, Option<(String, String)>)> = restaurantes::table
            .left_join(personas::table.on(restaurantes::encargado.eq(personas::id.nullable())))
            .select((
                RestauranteModel::as_select(),
                (personas::nombre, personas::apellidos).nullable()
            ))
            .order(restaurantes::nombre.asc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        
        let items: Vec<RestauranteListItemDto> = results
            .into_iter()
            .map(|(model, persona_data)| {
                let encargado_nombre = persona_data.map(|(nombre, apellidos)| format!("{} {}", nombre, apellidos));
                RestauranteListItemDto {
                    id: model.id,
                    nombre: model.nombre,
                    direccion: model.direccion,
                    telefono: model.telefono,
                    correo: model.correo,
                    tipo_atencion: model.tipo_atencion,
                    precio_promedio: model.precio_promedio,
                    capacidad: model.capacidad,
                    horario: model.horario,
                    encargado: model.encargado,
                    encargado_nombre,
                    is_active: model.is_active,
                    created_at: model.created_at,
                    updated_at: model.updated_at,
                }
            })
            .collect();
        
        let response = (items, total);
        
        // Store in cache
        if let Ok(serialized) = serde_json::to_string(&response) {
            self.cache.set_list(entity_names::RESTAURANTES, &cache_key, serialized).await;
        }
        
        info!("[CACHE MISS \u{2192} DB] restaurantes list '{}' ({} items, total: {}) | {}ms", cache_key, response.0.len(), total, start.elapsed().as_millis());
        Ok(response)
    }
}
