use std::sync::Arc;
use tracing::{info, debug};
use bigdecimal::BigDecimal;

use crate::application::ports::{EntradaPrecioRepositoryPort, CachePort, entity_names};
use crate::domain::entities::EntradaPrecio;
use crate::domain::errors::ApplicationError;

/// EntradaPrecioService - Servicio para la distribución de precios de entradas
/// 
/// Maneja la lógica de negocio para:
/// - CRUD de precios por rango de edad
/// - Cálculo de precio aplicable según edad y tipo de turista
/// - Caché para optimización de lecturas
pub struct EntradaPrecioService {
    entrada_precio_repository: Arc<dyn EntradaPrecioRepositoryPort>,
    cache: Arc<dyn CachePort>,
}

impl EntradaPrecioService {
    pub fn new(
        entrada_precio_repository: Arc<dyn EntradaPrecioRepositoryPort>,
        cache: Arc<dyn CachePort>,
    ) -> Self {
        Self {
            entrada_precio_repository,
            cache,
        }
    }

    /// Clave de caché para precios por entrada
    fn precios_cache_key(&self, id_entrada: i32) -> String {
        format!("entrada:{}", id_entrada)
    }

    // ==================== READ OPERATIONS ====================

    /// Obtener todos los precios de una entrada (con caché)
    pub async fn get_precios_by_entrada(&self, id_entrada: i32) -> Result<Vec<EntradaPrecio>, ApplicationError> {
        let cache_key = self.precios_cache_key(id_entrada);
        
        // Intentar obtener del caché
        if let Some(cached) = self.cache.get_list(entity_names::ENTRADA_PRECIOS, &cache_key).await {
            debug!("Cache HIT para entrada_precios by entrada: {}", id_entrada);
            if let Ok(response) = serde_json::from_str::<Vec<EntradaPrecio>>(&cached) {
                return Ok(response);
            }
        }
        
        debug!("Cache MISS para entrada_precios by entrada: {}", id_entrada);
        
        let result = self.entrada_precio_repository.find_by_entrada(id_entrada).await?;
        
        // Guardar en caché
        if let Ok(serialized) = serde_json::to_string(&result) {
            self.cache.set_list(entity_names::ENTRADA_PRECIOS, &cache_key, serialized).await;
        }
        
        Ok(result)
    }

    /// Obtener precios por entrada y tipo (general, nacional, extranjero)
    pub async fn get_precios_by_tipo(&self, id_entrada: i32, tipo_precio: &str) -> Result<Vec<EntradaPrecio>, ApplicationError> {
        self.entrada_precio_repository.find_by_entrada_and_tipo(id_entrada, tipo_precio).await
    }

    /// Obtener un precio específico por ID (con caché)
    pub async fn get_precio(&self, id: i32) -> Result<EntradaPrecio, ApplicationError> {
        // Intentar obtener del caché de detalle
        if let Some(cached) = self.cache.get_detail(entity_names::ENTRADA_PRECIOS, id).await {
            debug!("Cache HIT para entrada_precio: {}", id);
            if let Ok(response) = serde_json::from_str::<EntradaPrecio>(&cached) {
                return Ok(response);
            }
        }
        
        debug!("Cache MISS para entrada_precio: {}", id);
        
        let result = self.entrada_precio_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("Precio {} no encontrado", id)))?;
        
        // Guardar en caché
        if let Ok(serialized) = serde_json::to_string(&result) {
            self.cache.set_detail(entity_names::ENTRADA_PRECIOS, id, serialized).await;
        }
        
        Ok(result)
    }

    /// Calcular el precio aplicable para una entrada según edad y tipo de turista
    /// 
    /// Lógica de búsqueda:
    /// 1. Buscar precio con tipo_precio específico (nacional/extranjero)
    /// 2. Si no existe, buscar precio 'general'
    /// 3. Si no existe ninguno, retornar error
    pub async fn calcular_precio(
        &self,
        id_entrada: i32,
        edad: i32,
        tipo_turista: &str, // "nacional" o "extranjero"
    ) -> Result<BigDecimal, ApplicationError> {
        // Primero intentar con el tipo específico
        let precio = self.entrada_precio_repository
            .find_precio_for_edad(id_entrada, tipo_turista, edad)
            .await?;
        
        if let Some(p) = precio {
            return Ok(p.precio);
        }
        
        // Si no hay precio específico, buscar en 'general'
        let precio_general = self.entrada_precio_repository
            .find_precio_for_edad(id_entrada, "general", edad)
            .await?;
        
        if let Some(p) = precio_general {
            return Ok(p.precio);
        }
        
        Err(ApplicationError::NotFound(format!(
            "No se encontró precio para entrada {} con edad {} y tipo {}",
            id_entrada, edad, tipo_turista
        )))
    }

    // ==================== WRITE OPERATIONS ====================

    /// Invalidar caché de precios de una entrada específica
    async fn invalidate_precios_cache(&self, id_entrada: i32) {
        self.cache.invalidate_lists(entity_names::ENTRADA_PRECIOS).await;
        debug!("Cache de entrada_precios invalidado para entrada: {}", id_entrada);
    }

    /// Crear un nuevo precio de entrada
    pub async fn create_precio(
        &self,
        precio: &EntradaPrecio,
    ) -> Result<EntradaPrecio, ApplicationError> {
        // Validar que el rango de edad sea coherente
        if let Some(max) = precio.edad_maxima {
            if max < precio.edad_minima {
                return Err(ApplicationError::Validation(
                    "La edad máxima no puede ser menor que la edad mínima".to_string()
                ));
            }
        }
        
        let created = self.entrada_precio_repository.create(precio).await?;
        info!("Precio creado para entrada {}: {} ({}-{:?})", 
            created.id_entrada, created.tipo_precio, created.edad_minima, created.edad_maxima);
        
        // Invalidar caché
        self.invalidate_precios_cache(created.id_entrada).await;
        
        Ok(created)
    }

    /// Crear múltiples precios en batch
    pub async fn create_precios_batch(
        &self,
        precios: &[EntradaPrecio],
    ) -> Result<Vec<EntradaPrecio>, ApplicationError> {
        // Validar cada precio
        for precio in precios {
            if let Some(max) = precio.edad_maxima {
                if max < precio.edad_minima {
                    return Err(ApplicationError::Validation(format!(
                        "La edad máxima ({}) no puede ser menor que la edad mínima ({})",
                        max, precio.edad_minima
                    )));
                }
            }
        }
        
        let created = self.entrada_precio_repository.create_batch(precios).await?;
        info!("{} precios creados en batch", created.len());
        
        // Invalidar caché de todas las entradas afectadas
        if let Some(first) = created.first() {
            self.invalidate_precios_cache(first.id_entrada).await;
        }
        
        Ok(created)
    }

    /// Actualizar un precio existente
    pub async fn update_precio(
        &self,
        precio: &EntradaPrecio,
    ) -> Result<EntradaPrecio, ApplicationError> {
        // Validar rango de edad
        if let Some(max) = precio.edad_maxima {
            if max < precio.edad_minima {
                return Err(ApplicationError::Validation(
                    "La edad máxima no puede ser menor que la edad mínima".to_string()
                ));
            }
        }
        
        let updated = self.entrada_precio_repository.update(precio).await?;
        info!("Precio actualizado: ID {}", updated.id);
        
        // Invalidar caché
        self.invalidate_precios_cache(updated.id_entrada).await;
        self.cache.invalidate_detail(entity_names::ENTRADA_PRECIOS, updated.id).await;
        
        Ok(updated)
    }

    /// Eliminar un precio
    pub async fn delete_precio(&self, id: i32) -> Result<(), ApplicationError> {
        // Obtener el precio primero para saber su id_entrada
        let precio = self.get_precio(id).await?;
        let id_entrada = precio.id_entrada;
        
        if !self.entrada_precio_repository.delete(id).await? {
            return Err(ApplicationError::NotFound(format!("Precio {} no encontrado", id)));
        }
        info!("[DELETE] Precio eliminado: ID {}", id);
        
        // Invalidar caché
        self.invalidate_precios_cache(id_entrada).await;
        self.cache.invalidate_detail(entity_names::ENTRADA_PRECIOS, id).await;
        
        Ok(())
    }

    /// Reemplazar todos los precios de una entrada
    /// Útil para actualizar toda la estructura de precios de golpe
    pub async fn replace_all_precios(
        &self,
        id_entrada: i32,
        precios: &[EntradaPrecio],
    ) -> Result<Vec<EntradaPrecio>, ApplicationError> {
        // Validar cada precio
        for precio in precios {
            if precio.id_entrada != id_entrada {
                return Err(ApplicationError::Validation(
                    "Todos los precios deben pertenecer a la misma entrada".to_string()
                ));
            }
            if let Some(max) = precio.edad_maxima {
                if max < precio.edad_minima {
                    return Err(ApplicationError::Validation(format!(
                        "La edad máxima ({}) no puede ser menor que la edad mínima ({})",
                        max, precio.edad_minima
                    )));
                }
            }
        }
        
        let created = self.entrada_precio_repository.replace_all(id_entrada, precios).await?;
        info!("{} precios reemplazados para entrada {}", created.len(), id_entrada);
        
        // Invalidar caché
        self.invalidate_precios_cache(id_entrada).await;
        
        Ok(created)
    }

    /// Inicializar precios por defecto para una entrada nueva
    /// Crea estructura: general con 0-8 (gratis), 9-16, 17+ (adulto)
    pub async fn initialize_default_precios(
        &self,
        id_entrada: i32,
        created_by: Option<i32>,
    ) -> Result<Vec<EntradaPrecio>, ApplicationError> {
        let now = chrono::Utc::now();
        
        let default_precios = vec![
            EntradaPrecio {
                id: 0,
                id_entrada,
                tipo_precio: "general".to_string(),
                edad_minima: 0,
                edad_maxima: Some(8),
                precio: BigDecimal::from(0),
                descripcion: Some("Niño (gratis)".to_string()),
                created_at: now,
                updated_at: now,
                created_by,
                updated_by: created_by,
            },
            EntradaPrecio {
                id: 0,
                id_entrada,
                tipo_precio: "general".to_string(),
                edad_minima: 9,
                edad_maxima: Some(16),
                precio: BigDecimal::from(0),
                descripcion: Some("Adolescente".to_string()),
                created_at: now,
                updated_at: now,
                created_by,
                updated_by: created_by,
            },
            EntradaPrecio {
                id: 0,
                id_entrada,
                tipo_precio: "general".to_string(),
                edad_minima: 17,
                edad_maxima: None,
                precio: BigDecimal::from(0),
                descripcion: Some("Adulto".to_string()),
                created_at: now,
                updated_at: now,
                created_by,
                updated_by: created_by,
            },
        ];
        
        let created = self.entrada_precio_repository.create_batch(&default_precios).await?;
        
        // Invalidar caché
        self.invalidate_precios_cache(id_entrada).await;
        
        Ok(created)
    }
}
