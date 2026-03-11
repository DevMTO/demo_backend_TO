//! GET handlers para Auth

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tower_cookies::Cookies;
use tracing::{info, warn, debug, instrument};

use crate::application::dtos::auth_dto::{
    AuthUserInfo, PersonaProfileInfo, UserProfileResponse,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;

use super::helpers::create_session_cookie;

/// GET /api/v1/auth/verify - Verificar sesión actual
#[instrument(skip(state, cookies, auth_user))]
pub async fn verify_session_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("Verificando sesión para usuario: {}", auth_user.user.username);
    
    if let Some(new_token) = auth_user.rotated_token.as_ref() {
        debug!("Token rotado, actualizando cookie...");
        let session_cookie = create_session_cookie(
            new_token,
            state.container.cookie_max_age_hours * 3600,
            &state.container,
        );
        cookies.add(session_cookie);
    }
    
    let user_info = AuthUserInfo {
        id: auth_user.user.id,
        id_persona: auth_user.user.id_persona,
        username: auth_user.user.username.clone(),
        email: auth_user.user.email.clone(),
        role: auth_user.user.role.to_string(),
        id_entidad: auth_user.user.id_entidad,
        is_active: auth_user.user.is_active,
        turno: auth_user.user.turno.clone(),
    };
    
    info!("Sesión válida para: {}", auth_user.user.username);
    
    Ok((StatusCode::OK, Json(user_info)))
}

/// GET /health - Health check
pub async fn health_check() -> &'static str {
    "OK"
}

/// GET /api/v1/auth/profile - Obtener perfil completo del usuario
#[instrument(skip(state, auth_user))]
pub async fn get_profile_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("Obteniendo perfil para usuario: {}", auth_user.user.username);
    
    let user_info = AuthUserInfo {
        id: auth_user.user.id,
        id_persona: auth_user.user.id_persona,
        username: auth_user.user.username.clone(),
        email: auth_user.user.email.clone(),
        role: auth_user.user.role.to_string(),
        id_entidad: auth_user.user.id_entidad,
        is_active: auth_user.user.is_active,
        turno: auth_user.user.turno.clone(),
    };
    
    let persona_info = if let Some(id_persona) = auth_user.user.id_persona {
        match state.container.persona_repository.find_by_id(id_persona).await {
            Ok(Some(persona)) => Some(PersonaProfileInfo {
                id: persona.id,
                tipo_documento: persona.tipo_documento.to_string(),
                nro_documento: persona.nro_documento,
                nombre: persona.nombre,
                apellidos: persona.apellidos,
                telefono: persona.telefono,
                correo: persona.correo,
                fecha_nacimiento: persona.fecha_nacimiento,
            }),
            Ok(None) => {
                warn!("Persona con ID {} no encontrada para usuario {}", id_persona, auth_user.user.username);
                None
            },
            Err(e) => {
                warn!("Error al obtener persona {}: {}", id_persona, e);
                None
            }
        }
    } else {
        None
    };
    
    let response = UserProfileResponse {
        user: user_info,
        persona: persona_info,
    };
    
    info!("Perfil obtenido para: {}", auth_user.user.username);
    Ok((StatusCode::OK, Json(response)))
}
