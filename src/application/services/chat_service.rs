//! Chat Service - Manejo de notas/chats en files y file_tours
//! 
//! IMPORTANTE:
//! - file_tours.notas = JSONB (con mensajería, CRUD completo, SSE notificaciones)
//! - files.notas = TEXT (apartado separado, SIN notificaciones)

use std::sync::Arc;
use chrono::Utc;
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use crate::application::dtos::AuditInfo;
use crate::application::ports::{FileRepositoryPort, FileTourRepositoryPort};
use crate::domain::errors::ApplicationError;

pub struct ChatService {
    file_repository: Arc<dyn FileRepositoryPort>,
    file_tour_repository: Arc<dyn FileTourRepositoryPort>,
}

impl ChatService {
    pub fn new(
        file_repository: Arc<dyn FileRepositoryPort>,
        file_tour_repository: Arc<dyn FileTourRepositoryPort>,
    ) -> Self {
        Self {
            file_repository,
            file_tour_repository,
        }
    }

    /// Parse nota input - handles both plain string and JSON object
    fn parse_nota_input(input: &str) -> Option<JsonValue> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }
        
        // Try to parse as JSON first
        if let Ok(parsed) = serde_json::from_str::<JsonValue>(trimmed) {
            if parsed.is_object() {
                return Some(parsed);
            }
            if parsed.is_string() {
                return Some(json!({ "nota": parsed }));
            }
            return Some(json!({ "nota": parsed }));
        }
        
        Some(json!({ "nota": trimmed }))
    }

    /// Agrega una nota a un file (TEXT — apartado separado, SIN notificaciones)
    pub async fn chat_file(
        &self,
        file_id: i32,
        nota: Option<String>,
        user_info: Option<AuditInfo>,
    ) -> Result<Option<String>, ApplicationError> {
        let nota_value = match nota.as_ref().and_then(|n| Self::parse_nota_input(n)) {
            Some(n) if !n.is_null() => n,
            _ => return Ok(None),
        };

        let existing_file = self.file_repository
            .find_by_id(file_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;

        let existing_notas: JsonValue = existing_file.notas
            .as_ref()
            .and_then(|n| serde_json::from_str(n).ok())
            .unwrap_or(json!([]));

        let mut notas_array = if existing_notas.is_array() {
            existing_notas.as_array().unwrap().clone()
        } else {
            Vec::new()
        };

        let timestamp = Utc::now().to_rfc3339();
        let mut new_note = nota_value.clone();
        new_note["timestamp"] = json!(timestamp);

        if let Some(info) = user_info {
            new_note["user_id"] = json!(info.user_id);
            new_note["username"] = json!(info.username);
            new_note["is_admin"] = json!(info.is_admin);
        }

        notas_array.push(new_note);

        let updated_notas = serde_json::to_string(&notas_array).ok();

        let mut updated_file = existing_file;
        updated_file.notas = updated_notas.clone();
        self.file_repository.update(&updated_file).await?;

        Ok(updated_notas)
    }

    /// Agrega una nota a un file_tour (JSONB — con mensajería/notificaciones)
    ///
    /// El formato JSONB es: [{ timestamp, nota, user_id, username, is_admin, ... }]
    pub async fn chat_file_tour(
        &self,
        file_tour_id: i32,
        nota: Option<String>,
        user_info: Option<AuditInfo>,
    ) -> Result<Option<JsonValue>, ApplicationError> {
        let nota_value = match nota.as_ref().and_then(|n| Self::parse_nota_input(n)) {
            Some(n) if !n.is_null() => n,
            _ => return Ok(None),
        };

        let existing_file_tour = self.file_tour_repository
            .find_by_id(file_tour_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;

        // JSONB: usar directamente como JsonValue (ya no hay parsing de string)
        let existing_notas = existing_file_tour.notas.clone()
            .unwrap_or(json!([]));

        let mut notas_array = if existing_notas.is_array() {
            existing_notas.as_array().unwrap().clone()
        } else {
            Vec::new()
        };

        let timestamp = Utc::now().to_rfc3339();
        let note_id = Uuid::new_v4().to_string();
        let mut new_note = nota_value.clone();
        new_note["note_id"] = json!(note_id);
        new_note["timestamp"] = json!(timestamp);

        if let Some(info) = user_info {
            new_note["user_id"] = json!(info.user_id);
            new_note["username"] = json!(info.username);
            new_note["is_admin"] = json!(info.is_admin);
        }

        notas_array.push(new_note);

        let updated_notas = JsonValue::Array(notas_array);

        let mut updated_file_tour = existing_file_tour;
        updated_file_tour.notas = Some(updated_notas.clone());
        self.file_tour_repository.update(&updated_file_tour).await?;

        Ok(Some(updated_notas))
    }

    /// Actualiza una nota específica en un file_tour por note_id (JSONB)
    pub async fn update_note_file_tour(
        &self,
        file_tour_id: i32,
        note_id: &str,
        new_nota: &str,
        user_info: Option<AuditInfo>,
    ) -> Result<Option<JsonValue>, ApplicationError> {
        let existing_file_tour = self.file_tour_repository
            .find_by_id(file_tour_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;

        let existing_notas = existing_file_tour.notas.clone()
            .unwrap_or(json!([]));

        let mut notas_array = if existing_notas.is_array() {
            existing_notas.as_array().unwrap().clone()
        } else {
            return Err(ApplicationError::NotFound("Notas no encontradas".to_string()));
        };

        let note_index = notas_array.iter().position(|n| {
            n.get("note_id").and_then(|id| id.as_str()) == Some(note_id)
        }).ok_or_else(|| ApplicationError::NotFound(format!("Nota {} no encontrada", note_id)))?;

        // Verificar que el usuario es dueño de la nota o es admin
        if let Some(ref info) = user_info {
            if !info.is_admin {
                let note_user_id = notas_array[note_index].get("user_id").and_then(|v| v.as_i64());
                if note_user_id != Some(info.user_id as i64) {
                    return Err(ApplicationError::Forbidden("Solo puedes editar tus propias notas".to_string()));
                }
            }
        }

        notas_array[note_index]["nota"] = json!(new_nota);
        notas_array[note_index]["edited_at"] = json!(Utc::now().to_rfc3339());

        if let Some(info) = user_info {
            notas_array[note_index]["edited_by"] = json!(info.username);
        }

        let updated_notas = JsonValue::Array(notas_array);

        let mut updated_file_tour = existing_file_tour;
        updated_file_tour.notas = Some(updated_notas.clone());
        self.file_tour_repository.update(&updated_file_tour).await?;

        Ok(Some(updated_notas))
    }

    /// Elimina una nota específica de un file_tour por note_id (JSONB)
    pub async fn delete_note_file_tour(
        &self,
        file_tour_id: i32,
        note_id: &str,
        user_info: Option<AuditInfo>,
    ) -> Result<Option<JsonValue>, ApplicationError> {
        let existing_file_tour = self.file_tour_repository
            .find_by_id(file_tour_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;

        let existing_notas = existing_file_tour.notas.clone()
            .unwrap_or(json!([]));

        let mut notas_array = if existing_notas.is_array() {
            existing_notas.as_array().unwrap().clone()
        } else {
            return Err(ApplicationError::NotFound("Notas no encontradas".to_string()));
        };

        let note_index = notas_array.iter().position(|n| {
            n.get("note_id").and_then(|id| id.as_str()) == Some(note_id)
        }).ok_or_else(|| ApplicationError::NotFound(format!("Nota {} no encontrada", note_id)))?;

        // Verificar que el usuario es dueño de la nota o es admin
        if let Some(ref info) = user_info {
            if !info.is_admin {
                let note_user_id = notas_array[note_index].get("user_id").and_then(|v| v.as_i64());
                if note_user_id != Some(info.user_id as i64) {
                    return Err(ApplicationError::Forbidden("Solo puedes eliminar tus propias notas".to_string()));
                }
            }
        }

        notas_array.remove(note_index);

        let updated_notas = JsonValue::Array(notas_array);

        let mut updated_file_tour = existing_file_tour;
        updated_file_tour.notas = Some(updated_notas.clone());
        self.file_tour_repository.update(&updated_file_tour).await?;

        Ok(Some(updated_notas))
    }

    /// Obtiene las notas de un file (TEXT)
    pub async fn get_chat_file(&self, file_id: i32) -> Result<Option<String>, ApplicationError> {
        let file = self.file_repository
            .find_by_id(file_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;

        Ok(file.notas)
    }

    /// Obtiene las notas de un file_tour (JSONB)
    pub async fn get_chat_file_tour(&self, file_tour_id: i32) -> Result<Option<JsonValue>, ApplicationError> {
        let file_tour = self.file_tour_repository
            .find_by_id(file_tour_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;

        Ok(file_tour.notas)
    }
}
