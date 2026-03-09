use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{debug, info, instrument};

use crate::application::ports::{HotelRepositoryPort, CachePort, entity_names};
use crate::domain::{entities::Hotel, errors::ApplicationError};
use crate::infrastructure::persistence::{
    database::DatabasePool,
    models::{HotelModel, NewHotelModel, UpdateHotelModel},
    schema::{hoteles, cadenas_hoteleras, personas},
};

pub struct PostgresHotelRepository {
    pool: DatabasePool,
    cache: Arc<dyn CachePort>,
}

impl PostgresHotelRepository {
    pub fn new(pool: DatabasePool, cache: Arc<dyn CachePort>) -> Self {
        Self { pool, cache }
    }

    fn list_cache_key(limit: i64, offset: i64) -> String {
        format!("list:{}:{}", limit, offset)
    }

    async fn invalidate_cache(&self) {
        self.cache.invalidate_entity(entity_names::HOTELES).await;
        info!("[CACHE INVALIDATED] hoteles");
    }
}

#[async_trait]
impl HotelRepositoryPort for PostgresHotelRepository {
    #[instrument(skip(self, hotel))]
    async fn create(&self, hotel: &Hotel) -> Result<Hotel, ApplicationError> {
        debug!("Creando hotel: {}", hotel.nombre);
        let mut conn = self.pool.get_connection().await?;
        let new_hotel: NewHotelModel = hotel.into();

        let result = diesel::insert_into(hoteles::table)
            .values(&new_hotel)
            .get_result::<HotelModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        info!("Hotel creado: {} (id: {})", result.nombre, result.id);
        self.invalidate_cache().await;
        Ok(result.into())
    }

    #[instrument(skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<Option<Hotel>, ApplicationError> {
        let start = Instant::now();

        if let Some(cached) = self.cache.get_detail(entity_names::HOTELES, id).await {
            if let Ok(hotel) = serde_json::from_str::<Hotel>(&cached) {
                info!("[CACHE HIT] hotel #{} | {}ms", id, start.elapsed().as_millis());
                return Ok(Some(hotel));
            }
        }

        let mut conn = self.pool.get_connection().await?;
        let result = hoteles::table
            .filter(hoteles::id.eq(id))
            .first::<HotelModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        if let Some(ref model) = result {
            let hotel: Hotel = model.clone().into();
            if let Ok(serialized) = serde_json::to_string(&hotel) {
                self.cache.set_detail(entity_names::HOTELES, id, serialized).await;
            }
            info!("[CACHE MISS → DB] hotel #{} | {}ms", id, start.elapsed().as_millis());
            return Ok(Some(hotel));
        }

        Ok(None)
    }

    #[instrument(skip(self))]
    async fn find_by_encargado(&self, persona_id: i32) -> Result<Option<Hotel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let result = hoteles::table
            .filter(hoteles::encargado.eq(Some(persona_id)))
            .filter(hoteles::is_active.eq(true))
            .first::<HotelModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(result.map(Into::into))
    }

    #[instrument(skip(self, hotel))]
    async fn update(&self, hotel: &Hotel) -> Result<Hotel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let changes = UpdateHotelModel {
            id_cadena: Some(hotel.id_cadena),
            nombre: Some(&hotel.nombre),
            categoria: Some(hotel.categoria.as_deref()),
            telefono: Some(hotel.telefono.as_deref()),
            correo: Some(hotel.correo.as_deref()),
            direccion: Some(hotel.direccion.as_deref()),
            ciudad: Some(hotel.ciudad.as_deref()),
            media: Some(hotel.media.clone()),
            encargado: Some(hotel.encargado),
            is_active: Some(hotel.is_active),
            updated_by: hotel.updated_by,
        };
        let result = diesel::update(hoteles::table.filter(hoteles::id.eq(hotel.id)))
            .set(&changes)
            .get_result::<HotelModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        self.invalidate_cache().await;
        Ok(result.into())
    }

    async fn delete(&self, id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::delete(hoteles::table.filter(hoteles::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }

    async fn hard_delete(&self, id: i32) -> Result<bool, ApplicationError> {
        self.delete(id).await
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Hotel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = hoteles::table
            .filter(hoteles::is_active.eq(true))
            .order(hoteles::nombre.asc())
            .limit(limit)
            .offset(offset)
            .load::<HotelModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn count(&self) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        hoteles::table
            .filter(hoteles::is_active.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    async fn soft_delete(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(hoteles::table.filter(hoteles::id.eq(id)))
            .set((hoteles::is_active.eq(false), hoteles::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }

    async fn restore(&self, id: i32, user_id: i32) -> Result<bool, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let affected = diesel::update(hoteles::table.filter(hoteles::id.eq(id)))
            .set((hoteles::is_active.eq(true), hoteles::updated_by.eq(Some(user_id))))
            .execute(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        if affected > 0 { self.invalidate_cache().await; }
        Ok(affected > 0)
    }

    async fn list_by_cadena(&self, id_cadena: i32, limit: i64, offset: i64) -> Result<Vec<Hotel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        let results = hoteles::table
            .filter(hoteles::id_cadena.eq(id_cadena))
            .filter(hoteles::is_active.eq(true))
            .order(hoteles::nombre.asc())
            .limit(limit)
            .offset(offset)
            .load::<HotelModel>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn count_by_cadena(&self, id_cadena: i32) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        hoteles::table
            .filter(hoteles::id_cadena.eq(id_cadena))
            .filter(hoteles::is_active.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn list_with_cadena(&self, limit: i64, offset: i64) -> Result<(Vec<crate::application::dtos::HotelListItemDto>, i64), ApplicationError> {
        let start = Instant::now();
        let cache_key = Self::list_cache_key(limit, offset);

        if let Some(cached) = self.cache.get_list(entity_names::HOTELES, &cache_key).await {
            if let Ok(response) = serde_json::from_str::<(Vec<crate::application::dtos::HotelListItemDto>, i64)>(&cached) {
                info!("[CACHE HIT] hoteles list '{}' | {}ms", cache_key, start.elapsed().as_millis());
                return Ok(response);
            }
        }

        let mut conn = self.pool.get_connection().await?;

        let total: i64 = hoteles::table
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let results: Vec<(HotelModel, String, Option<(String, String)>)> = hoteles::table
            .inner_join(cadenas_hoteleras::table.on(hoteles::id_cadena.eq(cadenas_hoteleras::id)))
            .left_join(personas::table.on(hoteles::encargado.eq(personas::id.nullable())))
            .select((
                HotelModel::as_select(),
                cadenas_hoteleras::nombre,
                (personas::nombre, personas::apellidos).nullable(),
            ))
            .order(hoteles::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let items: Vec<crate::application::dtos::HotelListItemDto> = results
            .into_iter()
            .map(|(hotel, cadena_nombre, persona_data)| {
                let encargado_nombre = persona_data.map(|(nombre, apellidos)| {
                    format!("{} {}", nombre, apellidos)
                });

                crate::application::dtos::HotelListItemDto {
                    id: hotel.id,
                    id_cadena: hotel.id_cadena,
                    cadena_nombre: Some(cadena_nombre),
                    nombre: hotel.nombre,
                    categoria: hotel.categoria,
                    telefono: hotel.telefono,
                    correo: hotel.correo,
                    direccion: hotel.direccion,
                    ciudad: hotel.ciudad,
                    media: hotel.media,
                    encargado: hotel.encargado,
                    encargado_nombre,
                    is_active: hotel.is_active,
                    created_at: hotel.created_at,
                    updated_at: hotel.updated_at,
                }
            })
            .collect();

        let response = (items, total);
        if let Ok(serialized) = serde_json::to_string(&response) {
            self.cache.set_list(entity_names::HOTELES, &cache_key, serialized).await;
        }

        info!("[CACHE MISS → DB] hoteles list '{}' ({} items) | {}ms", cache_key, response.0.len(), start.elapsed().as_millis());
        Ok(response)
    }
}
