//! # Cache Port
//! 
//! Puerto (trait) para el sistema de caché de la aplicación.
//! Define la interface que debe implementar cualquier sistema de caché.

use async_trait::async_trait;

/// Puerto para operaciones de caché genérico por entidad
#[async_trait]
pub trait CachePort: Send + Sync {
    /// Obtener un valor del caché de lista por clave
    async fn get_list(&self, entity_type: &str, key: &str) -> Option<String>;
    
    /// Guardar un valor en el caché de lista
    async fn set_list(&self, entity_type: &str, key: &str, value: String);
    
    /// Obtener un valor del caché de detalle por ID
    async fn get_detail(&self, entity_type: &str, id: i32) -> Option<String>;
    
    /// Guardar un valor en el caché de detalle
    async fn set_detail(&self, entity_type: &str, id: i32, value: String);
    
    /// Invalidar todos los cachés de un tipo de entidad
    async fn invalidate_entity(&self, entity_type: &str);
    
    /// Invalidar un item específico del caché de detalle
    async fn invalidate_detail(&self, entity_type: &str, id: i32);
    
    /// Invalidar todo el caché de listas de un tipo de entidad
    async fn invalidate_lists(&self, entity_type: &str);
}

/// Nombres de entidades para el caché (constantes)
pub mod entity_names {
    pub const TOURS: &str = "tours";
    pub const ENTRADAS: &str = "entradas";
    pub const ENTRADA_PRECIOS: &str = "entrada_precios";
    pub const FILES: &str = "files";
    pub const AGENCIAS: &str = "agencias";
    pub const RESTAURANTES: &str = "restaurantes";
    pub const TRANSPORTES: &str = "transportes";
    pub const VEHICULOS: &str = "vehiculos";
    pub const CONDUCTORES: &str = "conductores";
    pub const GUIAS: &str = "guias";
    pub const PERSONAS: &str = "personas";
    pub const USERS: &str = "users";
    pub const MOVIMIENTOS: &str = "movimientos";
    pub const PAGOS: &str = "pagos";
}
