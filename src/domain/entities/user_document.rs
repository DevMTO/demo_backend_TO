//! # User Document Entity
//! 
//! Documento de identidad asociado a un usuario.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Documento de identidad de un usuario
/// 
/// Permite asociar múltiples documentos a un usuario,
/// separando la identidad del sistema de los documentos personales.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDocument {
    /// ID único del documento
    pub id: Uuid,
    /// ID del usuario al que pertenece
    pub user_id: Uuid,
    /// ID del tipo de documento
    pub document_type_id: i32,
    /// Número del documento
    pub document_number: String,
    /// Si es el documento principal
    pub is_primary: bool,
    /// Si ha sido verificado
    pub verified: bool,
    /// Fecha de verificación
    pub verified_at: Option<DateTime<Utc>>,
    /// Si está activo
    pub is_active: bool,
    /// Fecha de creación
    pub created_at: DateTime<Utc>,
    /// Fecha de actualización
    pub updated_at: DateTime<Utc>,
}

impl UserDocument {
    /// Crear un nuevo documento de usuario
    pub fn new(
        user_id: Uuid,
        document_type_id: i32,
        document_number: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            document_type_id,
            document_number,
            is_primary: false,
            verified: false,
            verified_at: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Crear como documento principal
    pub fn as_primary(mut self) -> Self {
        self.is_primary = true;
        self
    }
    
    /// Marcar como verificado
    pub fn verify(&mut self) {
        self.verified = true;
        self.verified_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    /// Establecer como documento principal
    pub fn set_as_primary(&mut self) {
        self.is_primary = true;
        self.updated_at = Utc::now();
    }
    
    /// Remover como documento principal
    pub fn unset_as_primary(&mut self) {
        self.is_primary = false;
        self.updated_at = Utc::now();
    }
    
    /// Desactivar documento
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }
}
