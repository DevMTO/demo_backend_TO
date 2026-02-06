//! # Módulo de Caché con Moka
//! 
//! Implementa un sistema de caché centralizado usando Moka para:
//! - Tours
//! - Entradas y EntradaPrecios
//! - Files y FileRelations
//! - Agencias, Restaurantes, etc.

use async_trait::async_trait;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::application::ports::{CachePort, entity_names};

/// Tiempo de expiración por defecto para entidades de uso frecuente (5 minutos)
const DEFAULT_TTL_SECS: u64 = 300;

/// Tiempo de expiración para listados (2 minutos)
const LIST_TTL_SECS: u64 = 120;

/// Tiempo de expiración para entidades poco frecuentes (10 minutos)
const LONG_TTL_SECS: u64 = 600;

/// Capacidad máxima del caché por tipo
const DEFAULT_MAX_CAPACITY: u64 = 1000;

/// Wrapper para valores en caché con metadata
#[derive(Clone, Debug)]
pub struct CachedValue<T> {
    pub value: T,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

impl<T: Clone> CachedValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            cached_at: chrono::Utc::now(),
        }
    }
}

/// Caché centralizado para la aplicación
pub struct AppCache {
    /// Caché para Tours (larga duración)
    pub tours_list: Cache<String, CachedValue<String>>,
    pub tours_detail: Cache<i32, CachedValue<String>>,
    
    /// Caché para Entradas (larga duración)
    pub entradas_list: Cache<String, CachedValue<String>>,
    pub entradas_detail: Cache<i32, CachedValue<String>>,
    
    /// Caché para EntradaPrecios (larga duración)
    pub entrada_precios_by_entrada: Cache<i32, CachedValue<String>>,
    pub entrada_precios_detail: Cache<i32, CachedValue<String>>,
    
    /// Caché para Files (duración media)
    pub files_list: Cache<String, CachedValue<String>>,
    pub files_detail: Cache<i32, CachedValue<String>>,
    
    /// Caché para FileRelations (duración media)
    pub file_relations_by_file: Cache<i32, CachedValue<String>>,
    
    /// Caché para Agencias (larga duración)
    pub agencias_list: Cache<String, CachedValue<String>>,
    pub agencias_detail: Cache<i32, CachedValue<String>>,
    
    /// Caché para Restaurantes (larga duración)
    pub restaurantes_list: Cache<String, CachedValue<String>>,
    pub restaurantes_detail: Cache<i32, CachedValue<String>>,
    
    /// Caché para Transportes (larga duración)
    pub transportes_list: Cache<String, CachedValue<String>>,
    pub transportes_detail: Cache<i32, CachedValue<String>>,
    
    /// Caché para Vehículos (larga duración)
    pub vehiculos_list: Cache<String, CachedValue<String>>,
    pub vehiculos_detail: Cache<i32, CachedValue<String>>,
    
    /// Caché para Conductores (larga duración)
    pub conductores_list: Cache<String, CachedValue<String>>,
    pub conductores_detail: Cache<i32, CachedValue<String>>,
    
    /// Caché para Guías (larga duración)
    pub guias_list: Cache<String, CachedValue<String>>,
    pub guias_detail: Cache<i32, CachedValue<String>>,
    
    /// Caché para Personas (larga duración)
    pub personas_list: Cache<String, CachedValue<String>>,
    pub personas_detail: Cache<i32, CachedValue<String>>,
    
    /// Caché para Usuarios (duración media)
    pub users_list: Cache<String, CachedValue<String>>,
    pub users_detail: Cache<i32, CachedValue<String>>,
    
    /// Caché para Contabilidad - Movimientos (corta duración)
    pub movimientos_list: Cache<String, CachedValue<String>>,
    
    /// Caché para Contabilidad - Pagos (corta duración)
    pub pagos_list: Cache<String, CachedValue<String>>,
    pub pagos_detail: Cache<i32, CachedValue<String>>,
}

impl AppCache {
    /// Crea una nueva instancia del caché centralizado
    pub fn new() -> Self {
        Self {
            // Tours - TTL largo (10 min)
            tours_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            tours_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            
            // Entradas - TTL largo (10 min)
            entradas_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            entradas_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            
            // EntradaPrecios - TTL largo (10 min)
            entrada_precios_by_entrada: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            entrada_precios_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            
            // Files - TTL medio (5 min)
            files_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(DEFAULT_TTL_SECS))
                .build(),
            files_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(DEFAULT_TTL_SECS))
                .build(),
            
            // FileRelations - TTL medio (5 min)
            file_relations_by_file: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(DEFAULT_TTL_SECS))
                .build(),
            
            // Agencias - TTL largo (10 min)
            agencias_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            agencias_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            
            // Restaurantes - TTL largo (10 min)
            restaurantes_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            restaurantes_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            
            // Transportes - TTL largo (10 min)
            transportes_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            transportes_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            
            // Vehículos - TTL largo (10 min)
            vehiculos_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            vehiculos_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            
            // Conductores - TTL largo (10 min)
            conductores_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            conductores_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            
            // Guías - TTL largo (10 min)
            guias_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            guias_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            
            // Personas - TTL largo (10 min)
            personas_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            personas_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LONG_TTL_SECS))
                .build(),
            
            // Users - TTL medio (5 min)
            users_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(DEFAULT_TTL_SECS))
                .build(),
            users_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(DEFAULT_TTL_SECS))
                .build(),
            
            // Movimientos - TTL corto (2 min)
            movimientos_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LIST_TTL_SECS))
                .build(),
            
            // Pagos - TTL corto (2 min)
            pagos_list: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LIST_TTL_SECS))
                .build(),
            pagos_detail: Cache::builder()
                .max_capacity(DEFAULT_MAX_CAPACITY)
                .time_to_live(Duration::from_secs(LIST_TTL_SECS))
                .build(),
        }
    }
    
    // ===================== INVALIDACIÓN DE CACHÉ =====================
    
    /// Invalida todo el caché de Tours
    pub async fn invalidate_tours(&self) {
        self.tours_list.invalidate_all();
        self.tours_detail.invalidate_all();
        tracing::debug!("Cache de tours invalidado");
    }
    
    /// Invalida un tour específico
    pub async fn invalidate_tour(&self, id: i32) {
        self.tours_list.invalidate_all();
        self.tours_detail.invalidate(&id).await;
        tracing::debug!("Cache de tour {} invalidado", id);
    }
    
    /// Invalida todo el caché de Entradas
    pub async fn invalidate_entradas(&self) {
        self.entradas_list.invalidate_all();
        self.entradas_detail.invalidate_all();
        tracing::debug!("Cache de entradas invalidado");
    }
    
    /// Invalida una entrada específica
    pub async fn invalidate_entrada(&self, id: i32) {
        self.entradas_list.invalidate_all();
        self.entradas_detail.invalidate(&id).await;
        tracing::debug!("Cache de entrada {} invalidado", id);
    }
    
    /// Invalida todo el caché de EntradaPrecios
    pub async fn invalidate_entrada_precios(&self) {
        self.entrada_precios_by_entrada.invalidate_all();
        self.entrada_precios_detail.invalidate_all();
        tracing::debug!("Cache de entrada_precios invalidado");
    }
    
    /// Invalida precios de una entrada específica
    pub async fn invalidate_entrada_precios_by_entrada(&self, id_entrada: i32) {
        self.entrada_precios_by_entrada.invalidate(&id_entrada).await;
        // También invalidamos el listado general
        self.entrada_precios_detail.invalidate_all();
        tracing::debug!("Cache de precios de entrada {} invalidado", id_entrada);
    }
    
    /// Invalida todo el caché de Files
    pub async fn invalidate_files(&self) {
        self.files_list.invalidate_all();
        self.files_detail.invalidate_all();
        self.file_relations_by_file.invalidate_all();
        tracing::debug!("Cache de files invalidado");
    }
    
    /// Invalida un file específico
    pub async fn invalidate_file(&self, id: i32) {
        self.files_list.invalidate_all();
        self.files_detail.invalidate(&id).await;
        self.file_relations_by_file.invalidate(&id).await;
        tracing::debug!("Cache de file {} invalidado", id);
    }
    
    /// Invalida todo el caché de Agencias
    pub async fn invalidate_agencias(&self) {
        self.agencias_list.invalidate_all();
        self.agencias_detail.invalidate_all();
        tracing::debug!("Cache de agencias invalidado");
    }
    
    /// Invalida una agencia específica
    pub async fn invalidate_agencia(&self, id: i32) {
        self.agencias_list.invalidate_all();
        self.agencias_detail.invalidate(&id).await;
        tracing::debug!("Cache de agencia {} invalidado", id);
    }
    
    /// Invalida todo el caché de Restaurantes
    pub async fn invalidate_restaurantes(&self) {
        self.restaurantes_list.invalidate_all();
        self.restaurantes_detail.invalidate_all();
        tracing::debug!("Cache de restaurantes invalidado");
    }
    
    /// Invalida un restaurante específico
    pub async fn invalidate_restaurante(&self, id: i32) {
        self.restaurantes_list.invalidate_all();
        self.restaurantes_detail.invalidate(&id).await;
        tracing::debug!("Cache de restaurante {} invalidado", id);
    }
    
    /// Invalida todo el caché de Transportes
    pub async fn invalidate_transportes(&self) {
        self.transportes_list.invalidate_all();
        self.transportes_detail.invalidate_all();
        tracing::debug!("Cache de transportes invalidado");
    }
    
    /// Invalida un transporte específico
    pub async fn invalidate_transporte(&self, id: i32) {
        self.transportes_list.invalidate_all();
        self.transportes_detail.invalidate(&id).await;
        tracing::debug!("Cache de transporte {} invalidado", id);
    }
    
    /// Invalida todo el caché de Vehículos
    pub async fn invalidate_vehiculos(&self) {
        self.vehiculos_list.invalidate_all();
        self.vehiculos_detail.invalidate_all();
        tracing::debug!("Cache de vehiculos invalidado");
    }
    
    /// Invalida un vehículo específico
    pub async fn invalidate_vehiculo(&self, id: i32) {
        self.vehiculos_list.invalidate_all();
        self.vehiculos_detail.invalidate(&id).await;
        tracing::debug!("Cache de vehiculo {} invalidado", id);
    }
    
    /// Invalida todo el caché de Conductores
    pub async fn invalidate_conductores(&self) {
        self.conductores_list.invalidate_all();
        self.conductores_detail.invalidate_all();
        tracing::debug!("Cache de conductores invalidado");
    }
    
    /// Invalida un conductor específico
    pub async fn invalidate_conductor(&self, id: i32) {
        self.conductores_list.invalidate_all();
        self.conductores_detail.invalidate(&id).await;
        tracing::debug!("Cache de conductor {} invalidado", id);
    }
    
    /// Invalida todo el caché de Guías
    pub async fn invalidate_guias(&self) {
        self.guias_list.invalidate_all();
        self.guias_detail.invalidate_all();
        tracing::debug!("Cache de guias invalidado");
    }
    
    /// Invalida un guía específico
    pub async fn invalidate_guia(&self, id: i32) {
        self.guias_list.invalidate_all();
        self.guias_detail.invalidate(&id).await;
        tracing::debug!("Cache de guia {} invalidado", id);
    }
    
    /// Invalida todo el caché de Personas
    pub async fn invalidate_personas(&self) {
        self.personas_list.invalidate_all();
        self.personas_detail.invalidate_all();
        tracing::debug!("Cache de personas invalidado");
    }
    
    /// Invalida una persona específica
    pub async fn invalidate_persona(&self, id: i32) {
        self.personas_list.invalidate_all();
        self.personas_detail.invalidate(&id).await;
        tracing::debug!("Cache de persona {} invalidado", id);
    }
    
    /// Invalida todo el caché de Users
    pub async fn invalidate_users(&self) {
        self.users_list.invalidate_all();
        self.users_detail.invalidate_all();
        tracing::debug!("Cache de users invalidado");
    }
    
    /// Invalida un user específico
    pub async fn invalidate_user(&self, id: i32) {
        self.users_list.invalidate_all();
        self.users_detail.invalidate(&id).await;
        tracing::debug!("Cache de user {} invalidado", id);
    }
    
    /// Invalida todo el caché de Movimientos
    pub async fn invalidate_movimientos(&self) {
        self.movimientos_list.invalidate_all();
        tracing::debug!("Cache de movimientos invalidado");
    }
    
    /// Invalida todo el caché de Pagos
    pub async fn invalidate_pagos(&self) {
        self.pagos_list.invalidate_all();
        self.pagos_detail.invalidate_all();
        tracing::debug!("Cache de pagos invalidado");
    }
    
    /// Invalida un pago específico
    pub async fn invalidate_pago(&self, id: i32) {
        self.pagos_list.invalidate_all();
        self.pagos_detail.invalidate(&id).await;
        tracing::debug!("Cache de pago {} invalidado", id);
    }
    
    /// Invalida todo el caché de la aplicación
    pub async fn invalidate_all(&self) {
        self.invalidate_tours().await;
        self.invalidate_entradas().await;
        self.invalidate_entrada_precios().await;
        self.invalidate_files().await;
        self.invalidate_agencias().await;
        self.invalidate_restaurantes().await;
        self.invalidate_transportes().await;
        self.invalidate_vehiculos().await;
        self.invalidate_conductores().await;
        self.invalidate_guias().await;
        self.invalidate_personas().await;
        self.invalidate_users().await;
        self.invalidate_movimientos().await;
        self.invalidate_pagos().await;
        tracing::info!("Todo el caché de la aplicación ha sido invalidado");
    }
    
    // ===================== ESTADÍSTICAS =====================
    
    /// Obtiene estadísticas del caché
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            tours_list_size: self.tours_list.entry_count(),
            tours_detail_size: self.tours_detail.entry_count(),
            entradas_list_size: self.entradas_list.entry_count(),
            entradas_detail_size: self.entradas_detail.entry_count(),
            files_list_size: self.files_list.entry_count(),
            files_detail_size: self.files_detail.entry_count(),
        }
    }
}

impl Default for AppCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas del caché
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub tours_list_size: u64,
    pub tours_detail_size: u64,
    pub entradas_list_size: u64,
    pub entradas_detail_size: u64,
    pub files_list_size: u64,
    pub files_detail_size: u64,
}

/// Helper para generar claves de caché para listados
pub fn list_cache_key(params: &impl Serialize) -> String {
    // Serializar los parámetros a JSON y usar como clave
    serde_json::to_string(params).unwrap_or_else(|_| "default".to_string())
}

// ===================== IMPLEMENTACIÓN DEL TRAIT CachePort =====================

#[async_trait]
impl CachePort for AppCache {
    async fn get_list(&self, entity_type: &str, key: &str) -> Option<String> {
        let key_owned = key.to_string();
        match entity_type {
            entity_names::TOURS => self.tours_list.get(&key_owned).await.map(|c| c.value.clone()),
            entity_names::ENTRADAS => self.entradas_list.get(&key_owned).await.map(|c| c.value.clone()),
            entity_names::FILES => self.files_list.get(&key_owned).await.map(|c| c.value.clone()),
            entity_names::AGENCIAS => self.agencias_list.get(&key_owned).await.map(|c| c.value.clone()),
            entity_names::RESTAURANTES => self.restaurantes_list.get(&key_owned).await.map(|c| c.value.clone()),
            entity_names::TRANSPORTES => self.transportes_list.get(&key_owned).await.map(|c| c.value.clone()),
            entity_names::VEHICULOS => self.vehiculos_list.get(&key_owned).await.map(|c| c.value.clone()),
            entity_names::CONDUCTORES => self.conductores_list.get(&key_owned).await.map(|c| c.value.clone()),
            entity_names::GUIAS => self.guias_list.get(&key_owned).await.map(|c| c.value.clone()),
            entity_names::PERSONAS => self.personas_list.get(&key_owned).await.map(|c| c.value.clone()),
            entity_names::USERS => self.users_list.get(&key_owned).await.map(|c| c.value.clone()),
            entity_names::MOVIMIENTOS => self.movimientos_list.get(&key_owned).await.map(|c| c.value.clone()),
            entity_names::PAGOS => self.pagos_list.get(&key_owned).await.map(|c| c.value.clone()),
            _ => None,
        }
    }

    async fn set_list(&self, entity_type: &str, key: &str, value: String) {
        let key_owned = key.to_string();
        let cached = CachedValue::new(value);
        match entity_type {
            entity_names::TOURS => self.tours_list.insert(key_owned, cached).await,
            entity_names::ENTRADAS => self.entradas_list.insert(key_owned, cached).await,
            entity_names::FILES => self.files_list.insert(key_owned, cached).await,
            entity_names::AGENCIAS => self.agencias_list.insert(key_owned, cached).await,
            entity_names::RESTAURANTES => self.restaurantes_list.insert(key_owned, cached).await,
            entity_names::TRANSPORTES => self.transportes_list.insert(key_owned, cached).await,
            entity_names::VEHICULOS => self.vehiculos_list.insert(key_owned, cached).await,
            entity_names::CONDUCTORES => self.conductores_list.insert(key_owned, cached).await,
            entity_names::GUIAS => self.guias_list.insert(key_owned, cached).await,
            entity_names::PERSONAS => self.personas_list.insert(key_owned, cached).await,
            entity_names::USERS => self.users_list.insert(key_owned, cached).await,
            entity_names::MOVIMIENTOS => self.movimientos_list.insert(key_owned, cached).await,
            entity_names::PAGOS => self.pagos_list.insert(key_owned, cached).await,
            _ => {}
        }
    }

    async fn get_detail(&self, entity_type: &str, id: i32) -> Option<String> {
        match entity_type {
            entity_names::TOURS => self.tours_detail.get(&id).await.map(|c| c.value.clone()),
            entity_names::ENTRADAS => self.entradas_detail.get(&id).await.map(|c| c.value.clone()),
            entity_names::ENTRADA_PRECIOS => self.entrada_precios_detail.get(&id).await.map(|c| c.value.clone()),
            entity_names::FILES => self.files_detail.get(&id).await.map(|c| c.value.clone()),
            entity_names::AGENCIAS => self.agencias_detail.get(&id).await.map(|c| c.value.clone()),
            entity_names::RESTAURANTES => self.restaurantes_detail.get(&id).await.map(|c| c.value.clone()),
            entity_names::TRANSPORTES => self.transportes_detail.get(&id).await.map(|c| c.value.clone()),
            entity_names::VEHICULOS => self.vehiculos_detail.get(&id).await.map(|c| c.value.clone()),
            entity_names::CONDUCTORES => self.conductores_detail.get(&id).await.map(|c| c.value.clone()),
            entity_names::GUIAS => self.guias_detail.get(&id).await.map(|c| c.value.clone()),
            entity_names::PERSONAS => self.personas_detail.get(&id).await.map(|c| c.value.clone()),
            entity_names::USERS => self.users_detail.get(&id).await.map(|c| c.value.clone()),
            entity_names::PAGOS => self.pagos_detail.get(&id).await.map(|c| c.value.clone()),
            _ => None,
        }
    }

    async fn set_detail(&self, entity_type: &str, id: i32, value: String) {
        let cached = CachedValue::new(value);
        match entity_type {
            entity_names::TOURS => self.tours_detail.insert(id, cached).await,
            entity_names::ENTRADAS => self.entradas_detail.insert(id, cached).await,
            entity_names::ENTRADA_PRECIOS => self.entrada_precios_detail.insert(id, cached).await,
            entity_names::FILES => self.files_detail.insert(id, cached).await,
            entity_names::AGENCIAS => self.agencias_detail.insert(id, cached).await,
            entity_names::RESTAURANTES => self.restaurantes_detail.insert(id, cached).await,
            entity_names::TRANSPORTES => self.transportes_detail.insert(id, cached).await,
            entity_names::VEHICULOS => self.vehiculos_detail.insert(id, cached).await,
            entity_names::CONDUCTORES => self.conductores_detail.insert(id, cached).await,
            entity_names::GUIAS => self.guias_detail.insert(id, cached).await,
            entity_names::PERSONAS => self.personas_detail.insert(id, cached).await,
            entity_names::USERS => self.users_detail.insert(id, cached).await,
            entity_names::PAGOS => self.pagos_detail.insert(id, cached).await,
            _ => {}
        }
    }

    async fn invalidate_entity(&self, entity_type: &str) {
        match entity_type {
            entity_names::TOURS => self.invalidate_tours().await,
            entity_names::ENTRADAS => self.invalidate_entradas().await,
            entity_names::ENTRADA_PRECIOS => self.invalidate_entrada_precios().await,
            entity_names::FILES => self.invalidate_files().await,
            entity_names::AGENCIAS => self.invalidate_agencias().await,
            entity_names::RESTAURANTES => self.invalidate_restaurantes().await,
            entity_names::TRANSPORTES => self.invalidate_transportes().await,
            entity_names::VEHICULOS => self.invalidate_vehiculos().await,
            entity_names::CONDUCTORES => self.invalidate_conductores().await,
            entity_names::GUIAS => self.invalidate_guias().await,
            entity_names::PERSONAS => self.invalidate_personas().await,
            entity_names::USERS => self.invalidate_users().await,
            entity_names::MOVIMIENTOS => self.invalidate_movimientos().await,
            entity_names::PAGOS => self.invalidate_pagos().await,
            _ => {}
        }
    }

    async fn invalidate_detail(&self, entity_type: &str, id: i32) {
        match entity_type {
            entity_names::TOURS => self.invalidate_tour(id).await,
            entity_names::ENTRADAS => self.invalidate_entrada(id).await,
            entity_names::ENTRADA_PRECIOS => {
                self.entrada_precios_detail.invalidate(&id).await;
            }
            entity_names::FILES => self.invalidate_file(id).await,
            entity_names::AGENCIAS => self.invalidate_agencia(id).await,
            entity_names::RESTAURANTES => self.invalidate_restaurante(id).await,
            entity_names::TRANSPORTES => self.invalidate_transporte(id).await,
            entity_names::VEHICULOS => self.invalidate_vehiculo(id).await,
            entity_names::CONDUCTORES => self.invalidate_conductor(id).await,
            entity_names::GUIAS => self.invalidate_guia(id).await,
            entity_names::PERSONAS => self.invalidate_persona(id).await,
            entity_names::USERS => self.invalidate_user(id).await,
            entity_names::PAGOS => self.invalidate_pago(id).await,
            _ => {}
        }
    }

    async fn invalidate_lists(&self, entity_type: &str) {
        match entity_type {
            entity_names::TOURS => self.tours_list.invalidate_all(),
            entity_names::ENTRADAS => self.entradas_list.invalidate_all(),
            entity_names::FILES => self.files_list.invalidate_all(),
            entity_names::AGENCIAS => self.agencias_list.invalidate_all(),
            entity_names::RESTAURANTES => self.restaurantes_list.invalidate_all(),
            entity_names::TRANSPORTES => self.transportes_list.invalidate_all(),
            entity_names::VEHICULOS => self.vehiculos_list.invalidate_all(),
            entity_names::CONDUCTORES => self.conductores_list.invalidate_all(),
            entity_names::GUIAS => self.guias_list.invalidate_all(),
            entity_names::PERSONAS => self.personas_list.invalidate_all(),
            entity_names::USERS => self.users_list.invalidate_all(),
            entity_names::MOVIMIENTOS => self.movimientos_list.invalidate_all(),
            entity_names::PAGOS => self.pagos_list.invalidate_all(),
            _ => {}
        }
    }
}
