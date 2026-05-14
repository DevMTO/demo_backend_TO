//! PUT handlers para Auth

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::{info, instrument};

use crate::application::dtos::auth_dto::{
    AuthUserInfo, UpdateProfileRequest,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;

/// PUT /api/v1/auth/profile - Actualizar perfil del usuario
#[instrument(skip(state, auth_user, payload))]
pub async fn update_profile_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("Actualizando perfil para usuario: {}", auth_user.user.username);
    
    // Actualizar la persona asociada si existe
    if let Some(id_persona) = auth_user.user.id_persona {
        if let Ok(Some(mut persona)) = state.container.persona_repository.find_by_id(id_persona).await {
            let mut updated = false;
            
            if let Some(nombre) = payload.nombre {
                persona.nombre = nombre;
                updated = true;
            }
            
            if let Some(apellidos) = payload.apellidos {
                persona.apellidos = apellidos;
                updated = true;
            }
            
            if let Some(telefono) = &payload.telefono {
                persona.telefono = Some(telefono.clone());
                updated = true;
            }
            
            if let Some(correo) = &payload.correo {
                persona.correo = Some(correo.clone());
                updated = true;
            }
            
            if let Some(fecha_nacimiento) = payload.fecha_nacimiento {
                persona.fecha_nacimiento = Some(fecha_nacimiento);
                updated = true;
            }
            
            if updated {
                state.container.persona_repository.update(&persona).await?;
            }
        }
    }
    
    let demo_expires_at = auth_user.user.demo_expires_at.map(|dt| dt.to_rfc3339());
    let user_info = AuthUserInfo {
        id: auth_user.user.id,
        id_persona: auth_user.user.id_persona,
        username: auth_user.user.username.clone(),
        email: auth_user.user.email.clone(),
        role: auth_user.user.role.to_string(),
        id_entidad: auth_user.user.id_entidad,
        is_active: auth_user.user.is_active,
        turno: auth_user.user.turno.clone(),
        is_demo: auth_user.user.is_demo,
        demo_expires_at,
    };
    
    info!("Perfil actualizado exitosamente");
    Ok((StatusCode::OK, Json(user_info)))
}
