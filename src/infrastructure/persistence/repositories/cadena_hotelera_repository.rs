use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{debug, info, instrument};

use crate::application::ports::{CadenaHoteleraRepositoryPort, CachePort, entity_names};
use crate::domain::{entities::CadenaHotelera, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{CadenaHoteleraModel, NewCadenaHoteleraModel, UpdateCadenaHoteleraModel},
    schema::{cadenas_hoteleras, hoteles},
};

pub struct PostgresCadenaHoteleraRepository {
    pool: DatabasePool,
    cache: Arc<dyn CachePort>,
}

impl PostgresCadenaHoteleraRepository {
    pub fn new(pool: DatabasePool, cache: Arc<dyn CachePort>) -> Self {
        Self { pool, cache }
    }

    fn list_cache_key(limit: i64, offset: i64) -> String {
        format!("list:{}:{}", limit, offset)
    }

    async fn invalidate_cache(&self) {
        self.cache.invalidate_entity(entity_names::CADENAS_HOTELERAS).await;
        info!("[CACHE INVALIDATED] cadenas_hoteleras");
    }
}

#[async_trait]
impl CadenaHoteleraRepositoryPort for PostgresCadenaHoteleraRepository {
    #[instrument(skip(self, cadena))]
    async fn create(&self, cadena: &CadenaHotelera) -> Result<CadenaHotelera, ApplicationError> {
        debug!("Creando cadena hotelera: {}", cadena.nombre);
        let mut conn = self.pool.get_connection().await?;
        let new_cadena: NewCadenaHoteleraModel = cadena.into();

        let result = diesel::insert_into(cadenas_hoteleras::table)
            .values(&new_cadena)
            .get_result::<CadenaHoteleraModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        info!("Cadena hotelera creada: {} (id: {})", result.nombre, result.id);
        self.invalidate_cache().await;
        Ok(result.into())
    }

    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<CadenaHotelera>, ApplicationError> {
        let start = Instant::now();

        if let Some(cached) = self.cache.get_detail(entity_names::CADENAS_HOTELERAS, id).await {
            if let Ok(cadena) = serde_json::from_str::<CadenaHotelera>(&cached) {
                info!("[CACHE HIT] cadena_hotelera #{} | {}ms", id, start.elapsed().as_millis());
                return Ok(Some(cadena));
            }
        }

        let mut conn = self.pool.get_connection().await?;
        let result = cadenas_hoteleras::table
            .filter(cadenas_hoteleras::id.eq(id))
            .first::<CadenaHoteleraModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        if let Some(ref model) = result {
            let cadena: CadenaHotelera = model.clone().into();
            if let Ok(serialized) = serde_json::to_string(&cadena) {
                self.cache.set_detail(entity_names::CADENAS_HOTELERAS, id, serialized).await;
            }
            info!("[CACHE MISS → DB] cadena_hotelera #{} | {}ms", id, start.elapsed().as_millis());
            return Ok(Some(cadena));
        }

        Ok(None)
    }

    #[instrument(skip(self))]
    async fn find_by_encargado(&self, persona_id: i32) -> Result<Option<CadenaHotelera>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = cadenas_hoteleras::table
            .filter(cadenas_hoteleras::encargado.eq(Some(persona_id)))
            .filter(cadenas_hoteleras::is_active.eq(true))
            .first::<CadenaHoteleraModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }

    #[instrument(skip(self, cadena))]
    async fn update(&self, cadena: &CadenaHotelera) -> Result<CadenaHotelera, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let changes = UpdateCadenaHoteleraModel {
            nombre: Some(&cadena.nombre),
            telefono: Some(cadena.telefono.as_deref()),
            correo: Some(cadena.correo.as_deref()),
            media: Some(cadena.media.clone()),
            encargado: Some(cadena.encargado),
            is_active: Some(cadena.is_active),
            paleta_colores: Some(cadena.paleta_colores.clone()),
            updated_by: cadena.updated_by,
        };
        let result = diesel::update(cadenas_hoteleras::table.filter(cadenas_hoteleras::id.eq(cadena.id)))
            .set(&changes)
            .get_result::<CadenaHoteleraModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        self.invalidate_cache().await;
        Ok(result.into())
    }

    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(cadenas_hoteleras::table.filter(cadenas_hoteleras::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }

    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError> {
        self.delete(id).await
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<CadenaHotelera>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = cadenas_hoteleras::table
            .filter(cadenas_hoteleras::is_active.eq(true))
            .order(cadenas_hoteleras::nombre.asc())
            .limit(limit)
            .offset(offset)
            .load::<CadenaHoteleraModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn count(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        cadenas_hoteleras::table
            .filter(cadenas_hoteleras::is_active.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(cadenas_hoteleras::table.filter(cadenas_hoteleras::id.eq(id)))
            .set((cadenas_hoteleras::is_active.eq(false), cadenas_hoteleras::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }

    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(cadenas_hoteleras::table.filter(cadenas_hoteleras::id.eq(id)))
            .set((cadenas_hoteleras::is_active.eq(true), cadenas_hoteleras::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }

    #[instrument(skip(self))]
    async fn list_with_encargado(&self, limit: i64, offset: i64) -> Result<(Vec<crate::application::dtos::CadenaHoteleraListItemDto>, i64), ApplicationError> {
        use crate::infrastructure::persistence::schema::personas;

        let start = Instant::now();
        let cache_key = Self::list_cache_key(limit, offset);

        if let Some(cached) = self.cache.get_list(entity_names::CADENAS_HOTELERAS, &cache_key).await {
            if let Ok(response) = serde_json::from_str::<(Vec<crate::application::dtos::CadenaHoteleraListItemDto>, i64)>(&cached) {
                info!("[CACHE HIT] cadenas_hoteleras list '{}' | {}ms", cache_key, start.elapsed().as_millis());
                return Ok(response);
            }
        }

        let mut conn = self.pool.get_connection().await?;

        let total: i64 = cadenas_hoteleras::table
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let results: Vec<(CadenaHoteleraModel, Option<(String, String)>)> = cadenas_hoteleras::table
            .left_join(personas::table.on(cadenas_hoteleras::encargado.eq(personas::id.nullable())))
            .select((
                CadenaHoteleraModel::as_select(),
                (personas::nombre, personas::apellidos).nullable(),
            ))
            .order(cadenas_hoteleras::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // Count hoteles per cadena
        let cadena_ids: Vec<i32> = results.iter().map(|(c, _)| c.id).collect();
        let hotel_counts: Vec<(i32, i64)> = if !cadena_ids.is_empty() {
            hoteles::table
                .filter(hoteles::id_cadena.eq_any(&cadena_ids))
                .filter(hoteles::is_active.eq(true))
                .group_by(hoteles::id_cadena)
                .select((hoteles::id_cadena, diesel::dsl::count(hoteles::id)))
                .load(&mut conn)
                .await
                .map_err(|e| ApplicationError::Repository(e.to_string()))?
        } else {
            vec![]
        };

        let items: Vec<crate::application::dtos::CadenaHoteleraListItemDto> = results
            .into_iter()
            .map(|(cadena, persona_data)| {
                let encargado_nombre = persona_data.map(|(nombre, apellidos)| {
                    format!("{} {}", nombre, apellidos)
                });
                let total_hoteles = hotel_counts.iter()
                    .find(|(cid, _)| *cid == cadena.id)
                    .map(|(_, count)| *count)
                    .unwrap_or(0);

                crate::application::dtos::CadenaHoteleraListItemDto {
                    id: cadena.id,
                    nombre: cadena.nombre,
                    telefono: cadena.telefono,
                    correo: cadena.correo,
                    media: cadena.media,
                    encargado: cadena.encargado,
                    encargado_nombre,
                    is_active: cadena.is_active,
                    paleta_colores: cadena.paleta_colores,
                    total_hoteles,
                    created_at: cadena.created_at,
                    updated_at: cadena.updated_at,
                }
            })
            .collect();

        let response = (items, total);
        if let Ok(serialized) = serde_json::to_string(&response) {
            self.cache.set_list(entity_names::CADENAS_HOTELERAS, &cache_key, serialized).await;
        }

        info!("[CACHE MISS → DB] cadenas_hoteleras list '{}' ({} items) | {}ms", cache_key, response.0.len(), start.elapsed().as_millis());
        Ok(response)
    }
}
