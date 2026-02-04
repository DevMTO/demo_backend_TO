//! GET handlers para Mis Pagos
//!
//! Endpoints para que los proveedores vean sus pagos

use axum::{
    extract::State,
    response::IntoResponse,
};
use tracing::{info, instrument};

use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;

// ============================================================================
// MIS PAGOS - GUIA
// ============================================================================

/// GET /api/v1/mis-pagos/guia
/// Obtiene los pagos para el guía autenticado
#[instrument(skip(state, auth))]
pub async fn get_mis_pagos_guia(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario es un guía
    if auth.user.role != UserRole::Guias {
        return Err(ApplicationError::Forbidden(
            "Solo los guías pueden acceder a este endpoint".to_string(),
        ));
    }
    
    // Verificar que el usuario tiene id_persona
    let id_persona = auth.user.id_persona
        .ok_or_else(|| ApplicationError::Validation(
            "Usuario no tiene persona asociada".to_string()
        ))?;
    
    info!("Consultando mis pagos para guía con id_persona: {}", id_persona);
    
    let pagos = state
        .container
        .mis_pagos_service
        .get_mis_pagos_guia(id_persona)
        .await?;
    
    info!("Encontrados {} pagos para guía id_persona: {}", pagos.len(), id_persona);
    
    Ok(json_ok(pagos))
}

// ============================================================================
// MIS PAGOS - CONDUCTOR
// ============================================================================

/// GET /api/v1/mis-pagos/conductor
/// Obtiene los pagos para el conductor autenticado
#[instrument(skip(state, auth))]
pub async fn get_mis_pagos_conductor(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario es un conductor
    if auth.user.role != UserRole::Conductores {
        return Err(ApplicationError::Forbidden(
            "Solo los conductores pueden acceder a este endpoint".to_string(),
        ));
    }
    
    // Verificar que el usuario tiene id_persona
    let id_persona = auth.user.id_persona
        .ok_or_else(|| ApplicationError::Validation(
            "Usuario no tiene persona asociada".to_string()
        ))?;
    
    info!("Consultando mis pagos para conductor con id_persona: {}", id_persona);
    
    let pagos = state
        .container
        .mis_pagos_service
        .get_mis_pagos_conductor(id_persona)
        .await?;
    
    info!("Encontrados {} pagos para conductor id_persona: {}", pagos.len(), id_persona);
    
    Ok(json_ok(pagos))
}

// ============================================================================
// MIS PAGOS - RESTAURANTE
// ============================================================================

/// GET /api/v1/mis-pagos/restaurante
/// Obtiene los pagos para el restaurante autenticado
#[instrument(skip(state, auth))]
pub async fn get_mis_pagos_restaurante(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que el usuario es un restaurante
    if auth.user.role != UserRole::Restaurantes {
        return Err(ApplicationError::Forbidden(
            "Solo los restaurantes pueden acceder a este endpoint".to_string(),
        ));
    }
    
    // Verificar que el usuario tiene id_entidad (que es el id_restaurante)
    let id_restaurante = auth.user.id_entidad
        .ok_or_else(|| ApplicationError::Validation(
            "Usuario no tiene restaurante asociado".to_string()
        ))?;
    
    info!("Consultando mis pagos para restaurante con id: {}", id_restaurante);
    
    let pagos = state
        .container
        .mis_pagos_service
        .get_mis_pagos_restaurante(id_restaurante)
        .await?;
    
    info!("Encontrados {} pagos para restaurante id: {}", pagos.len(), id_restaurante);
    
    Ok(json_ok(pagos))
}
