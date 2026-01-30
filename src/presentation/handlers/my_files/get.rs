//! GET handlers para My Files
//! Endpoints para que usuarios vean sus files asignados según su rol

use axum::{extract::State, response::IntoResponse};
use tracing::{error, info, instrument};

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;

/// GET /api/v1/my-files/guia - Obtiene los files asignados al guía autenticado
/// Requiere que el usuario tenga id_persona y sea guía
#[instrument(skip(state, auth))]
pub async fn get_my_files_as_guia(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario tiene id_persona
    let id_persona = auth.user.id_persona
        .ok_or_else(|| ApplicationError::Validation("Usuario no tiene persona asociada".to_string()))?;
    
    // Verificar que el usuario es un guía
    if auth.user.role != UserRole::Guias {
        return Err(ApplicationError::Forbidden("Solo los guías pueden acceder a este endpoint".to_string()));
    }
    
    info!("Consultando files para guía con id_persona: {}", id_persona);
    
    let files = match state.container.my_files_service.get_my_files_as_guia(id_persona).await {
        Ok(files) => {
            info!("Encontrados {} files para guía id_persona: {}", files.len(), id_persona);
            files
        },
        Err(e) => {
            error!("Error consultando files para guía id_persona {}: {:?}", id_persona, e);
            return Err(e);
        }
    };
    
    Ok(json_ok(files))
}

/// GET /api/v1/my-files/conductor - Obtiene los files asignados al conductor autenticado
/// Requiere que el usuario tenga id_persona y sea conductor
#[instrument(skip(state, auth))]
pub async fn get_my_files_as_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario tiene id_persona
    let id_persona = auth.user.id_persona
        .ok_or_else(|| ApplicationError::Validation("Usuario no tiene persona asociada".to_string()))?;
    
    // Verificar que el usuario es un conductor
    if auth.user.role != UserRole::Conductores {
        return Err(ApplicationError::Forbidden("Solo los conductores pueden acceder a este endpoint".to_string()));
    }
    
    let files = state.container.my_files_service.get_my_files_as_conductor(id_persona).await?;
    Ok(json_ok(files))
}

/// GET /api/v1/my-files/restaurante - Obtiene los files asignados al restaurante autenticado
/// Requiere que el usuario tenga role restaurantes y id_entidad configurado
#[instrument(skip(state, auth))]
pub async fn get_my_files_as_restaurante(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario es un restaurante
    if auth.user.role != UserRole::Restaurantes {
        return Err(ApplicationError::Forbidden("Solo los restaurantes pueden acceder a este endpoint".to_string()));
    }
    
    // Obtener el id_restaurante desde id_entidad del usuario
    let id_restaurante = auth.user.id_entidad
        .ok_or_else(|| ApplicationError::Validation("Usuario no tiene restaurante asociado (id_entidad)".to_string()))?;
    
    let files = state.container.my_files_service.get_my_files_as_restaurante(id_restaurante).await?;
    Ok(json_ok(files))
}
