//! Chat Service - Manejo de notas/chats en files y file_tours
//! 
//! Proporciona funcionalidades para agregar notas a files y file_tours
//! de forma estructurada como JSON array: [{ timestamp, ...noteData }, ...]

use std::sync::Arc;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::application::ports::{FileRepositoryPort, FileTourRepositoryPort};
use crate::domain::errors::ApplicationError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatUserInfo {
    pub user_id: i32,
    pub username: String,
    pub is_admin: bool,
}

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
    /// If input is a valid JSON object, returns it as-is
    /// If input is a plain string, wraps it as { "nota": "string" }
    fn parse_nota_input(input: &str) -> Option<JsonValue> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }
        
        // Try to parse as JSON first
        if let Ok(parsed) = serde_json::from_str::<JsonValue>(trimmed) {
            // If it's an object, use it as-is
            if parsed.is_object() {
                return Some(parsed);
            }
            // If it's a string, wrap it
            if parsed.is_string() {
                return Some(json!({ "nota": parsed }));
            }
            // For other types (number, bool, etc), wrap as nota
            return Some(json!({ "nota": parsed }));
        }
        
        // If parsing fails, treat it as a plain string
        Some(json!({ "nota": trimmed }))
    }

    /// Agrega una nota a un file
    /// 
    /// Si `nota` es None o vacía, no hace nada.
    /// Si el file tiene notas existentes, las mantiene y agrega la nueva.
    /// El formato de las notas es: [{ timestamp, ...noteData }, ...]
    pub async fn chat_file(
        &self,
        file_id: i32,
        nota: Option<String>,
        user_info: Option<ChatUserInfo>,
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

    /// Agrega una nota a un file_tour
    /// 
    /// Si `nota` es None o vacía, no hace nada.
    /// Si el file_tour tiene notas existentes, las mantiene y agrega la nueva.
    /// El formato de las notas es: [{ timestamp, ...noteData }, ...]
    pub async fn chat_file_tour(
        &self,
        file_tour_id: i32,
        nota: Option<String>,
        user_info: Option<ChatUserInfo>,
    ) -> Result<Option<String>, ApplicationError> {
        let nota_value = match nota.as_ref().and_then(|n| Self::parse_nota_input(n)) {
            Some(n) if !n.is_null() => n,
            _ => return Ok(None),
        };

        let existing_file_tour = self.file_tour_repository
            .find_by_id(file_tour_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;

        let existing_notas: JsonValue = existing_file_tour.notas
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

        let mut updated_file_tour = existing_file_tour;
        updated_file_tour.notas = updated_notas.clone();
        self.file_tour_repository.update(&updated_file_tour).await?;

        Ok(updated_notas)
    }

    /// Obtiene las notas de un file
    pub async fn get_chat_file(&self, file_id: i32) -> Result<Option<String>, ApplicationError> {
        let file = self.file_repository
            .find_by_id(file_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;

        Ok(file.notas)
    }

    /// Obtiene las notas de un file_tour
    pub async fn get_chat_file_tour(&self, file_tour_id: i32) -> Result<Option<String>, ApplicationError> {
        let file_tour = self.file_tour_repository
            .find_by_id(file_tour_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;

        Ok(file_tour.notas)
    }
}
