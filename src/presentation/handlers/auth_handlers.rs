use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use tower_cookies::Cookies;
use tracing::{info, warn, debug, instrument};
use validator::Validate;

use crate::application::dtos::auth_dto::{
    LoginRequest, LogoutRequest, AuthResponse,
    SuccessResponse, AuthUserInfo,
    PersonaProfileInfo, UserProfileResponse, UpdateProfileRequest,
};
use crate::domain::entities::{
    NotificationType, NotificationCategory, NotificationPriority, UserRole,
};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;

#[instrument(skip(state, cookies, request), fields(identifier = %request.identifier))]
pub async fn login_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("Intento de login para: {} (remember_me: {})", request.identifier, request.remember_me);
    
    // Extraer IP y User-Agent del request
    let ip_address = None;
    let user_agent = None;
    
    // Ejecutar caso de uso
    debug!("Ejecutando LoginUseCase (remember_me: {})...", request.remember_me);
    let output = match state.container.login_use_case
        .execute(request.clone(), ip_address.clone(), user_agent.clone())
        .await {
            Ok(output) => {
                info!("Login exitoso para usuario: {} (id: {}, remember_me: {})", output.user_info.username, output.user_info.id, request.remember_me);
                
                // Logging del login exitoso
                if let Err(e) = state.container.logging_service.log_login(
                    output.user_info.id,
                    &output.user_info.username,
                    ip_address.clone(),
                    user_agent.clone(),
                ).await {
                    warn!("Error al registrar log de login: {}", e);
                }
                
                output
            },
            Err(e) => {
                warn!("Login fallido para {}: {:?}", request.identifier, e);
                
                // Logging del login fallido
                if let Err(log_err) = state.container.logging_service.log_login_failed(
                    &request.identifier,
                    &e.to_string(),
                    ip_address,
                    user_agent,
                ).await {
                    warn!("Error al registrar log de login fallido: {}", log_err);
                }
                
                // Si el error es por cuenta bloqueada, notificar a admins
                if matches!(e, ApplicationError::Authentication(_)) && e.to_string().contains("bloqueada") {
                    // Intentar obtener el user_id del identifier para notificar
                    if let Ok(Some(blocked_user)) = state.container.user_repository
                        .find_by_username(&request.identifier)
                        .await 
                    {
                        // Notificar a admins sobre cuenta bloqueada via SSE broadcast
                        if let Err(notif_err) = state.notify_roles_with_broadcast(
                            vec![UserRole::SuperAdmin, UserRole::Admin],
                            "Cuenta bloqueada por intentos fallidos",
                            &format!("El usuario '{}' ha sido bloqueado por superar el máximo de intentos de login", blocked_user.username),
                            NotificationType::Warning,
                            NotificationCategory::Auth,
                            NotificationPriority::High,
                            None,
                        ).await {
                            warn!("Error al notificar bloqueo de cuenta: {}", notif_err);
                        }
                    }
                }
                
                return Err(e);
            }
        };
    
    // Configurar cookie de sesión
    debug!("Configurando cookie de sesión...");
    debug!("Cookie config - name: {}, path: {}, http_only: {}, secure: {}, same_site: {}", 
        state.container.cookie_name,
        state.container.cookie_path,
        state.container.cookie_http_only,
        state.container.cookie_secure,
        state.container.cookie_same_site
    );
    
    let session_cookie = create_session_cookie(
        &output.session_token,
        output.expires_in_seconds,
        &state.container,
    );
    
    info!("🍪 Cookie creada: name={}, max_age={}s", 
        state.container.cookie_name, 
        output.expires_in_seconds
    );
    
    cookies.add(session_cookie);
    
    // Construir respuesta
    let auth_response = AuthResponse::new(
        output.user_info,
        output.session_id,
        output.expires_in_seconds,
        request.remember_me,
    );
    
    info!("🎉 Login completo, sesión creada: {}", output.session_id);
    
    Ok((StatusCode::OK, Json(auth_response)))
}

#[instrument(skip(state, cookies, auth_user, request))]
pub async fn logout_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    auth_user: AuthUser,
    Json(request): Json<LogoutRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("🚪 Logout para usuario: {} (sesión: {})", auth_user.user.username, auth_user.session_id);
    
    // Ejecutar caso de uso
    let count = state.container.logout_use_case
        .execute(auth_user.user.id, auth_user.session_id, request.clone())
        .await?;
    
    // Limpiar cookie de sesión
    remove_session_cookie(&cookies, &state.container);
    
    info!("Logout completado: {} sesión(es) cerrada(s)", count);
    
    // Logging del logout con información detallada
    if let Err(e) = state.container.logging_service.log_logout(
        auth_user.user.id,
        &auth_user.user.username,
        auth_user.session_id,
        None,
        request.all_sessions,
        count,
    ).await {
        warn!("Error al registrar log de logout: {}", e);
    }
    
    Ok((
        StatusCode::OK,
        Json(SuccessResponse::new(format!("{} sesión(es) cerrada(s)", count))),
    ))
}

#[instrument(skip(state, cookies, auth_user))]
pub async fn verify_session_handler(
    State(state): State<AppState>,
    cookies: Cookies,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("Verificando sesión para usuario: {}", auth_user.user.username);
    
    // Si la sesión fue rotada, actualizar la cookie
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
    };
    
    info!("Sesión válida para: {}", auth_user.user.username);
    
    Ok((StatusCode::OK, Json(user_info)))
}

pub async fn health_check() -> &'static str {
    "OK"
}

/// Handler para obtener el perfil completo del usuario autenticado (incluyendo persona)
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
    };
    
    // Obtener la persona asociada si existe
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

/// Handler para actualizar el perfil del usuario autenticado
#[instrument(skip(state, auth_user, request))]
pub async fn update_profile_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    info!("Actualizando perfil para usuario: {}", auth_user.user.username);
    
    // Validar request
    request.validate()
        .map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    // Verificar que el usuario tenga persona asociada
    let id_persona = auth_user.user.id_persona
        .ok_or_else(|| ApplicationError::BadRequest("Usuario no tiene persona asociada".to_string()))?;
    
    // Convertir a UpdatePersonaRequest y usar el servicio
    let update_request = crate::application::dtos::UpdatePersonaRequest {
        nombre: request.nombre,
        apellidos: request.apellidos,
        tipo_documento: None,
        nro_documento: None,
        telefono: request.telefono,
        correo: request.correo,
        fecha_nacimiento: request.fecha_nacimiento,
    };
    
    // Usar PersonaService (que incluye logging)
    let updated = state.container.persona_service
        .update_persona(
            id_persona,
            update_request,
            auth_user.user.id,
            Some(auth_user.user.username.clone()),
        )
        .await?;
    
    // Construir respuesta
    let user_info = AuthUserInfo {
        id: auth_user.user.id,
        id_persona: auth_user.user.id_persona,
        username: auth_user.user.username.clone(),
        email: auth_user.user.email.clone(),
        role: auth_user.user.role.to_string(),
        id_entidad: auth_user.user.id_entidad,
        is_active: auth_user.user.is_active,
    };
    
    let persona_info = PersonaProfileInfo {
        id: updated.id,
        tipo_documento: updated.tipo_documento,
        nro_documento: updated.nro_documento,
        nombre: updated.nombre,
        apellidos: updated.apellidos,
        telefono: updated.telefono,
        correo: updated.correo,
        fecha_nacimiento: updated.fecha_nacimiento,
    };
    
    let response = UserProfileResponse {
        user: user_info,
        persona: Some(persona_info),
    };
    
    info!("Perfil actualizado para: {}", auth_user.user.username);
    Ok((StatusCode::OK, Json(response)))
}

fn create_session_cookie(
    token: &str,
    max_age_secs: i64,
    container: &crate::infrastructure::container::DependencyContainer,
) -> Cookie<'static> {
    // Para desarrollo local (HTTP) con cross-origin (frontend en 3000, backend en 8080):
    // - SameSite=Lax permite que las cookies se envíen en navegación top-level
    // - SameSite=None requiere Secure=true (solo HTTPS)
    // 
    // En producción (HTTPS):
    // - Usar SameSite=Strict o SameSite=None con Secure=true
    let same_site = match container.cookie_same_site.to_lowercase().as_str() {
        "lax" => SameSite::Lax,
        "none" => SameSite::None,
        _ => SameSite::Strict,
    };
    
    // Construir cookie con todos los atributos de seguridad
    let mut cookie_builder = Cookie::build((container.cookie_name.clone(), token.to_string()))
        .path(container.cookie_path.clone())
        .http_only(container.cookie_http_only)
        .same_site(same_site)
        .max_age(time::Duration::seconds(max_age_secs));
    
    // Solo agregar Secure si está habilitado (producción con HTTPS)
    // En desarrollo con HTTP, no se debe agregar Secure porque el navegador rechazaría la cookie
    if container.cookie_secure {
        cookie_builder = cookie_builder.secure(true);
    }
    
    // Agregar domain si está configurado y no es localhost (en desarrollo no se necesita)
    if !container.cookie_domain.is_empty() 
        && container.cookie_domain != "localhost" 
        && container.cookie_domain != "127.0.0.1" 
    {
        cookie_builder = cookie_builder.domain(container.cookie_domain.clone());
    }
    
    debug!("🍪 Cookie settings: name={}, path={}, http_only={}, secure={}, same_site={:?}, max_age={}s, domain={}",
        container.cookie_name,
        container.cookie_path,
        container.cookie_http_only,
        container.cookie_secure,
        same_site,
        max_age_secs,
        container.cookie_domain
    );
    
    cookie_builder.build()
}

fn remove_session_cookie(
    cookies: &Cookies,
    container: &crate::infrastructure::container::DependencyContainer,
) {
    let same_site = match container.cookie_same_site.to_lowercase().as_str() {
        "lax" => SameSite::Lax,
        "none" => SameSite::None,
        _ => SameSite::Strict,
    };
    
    let mut cookie_builder = Cookie::build((container.cookie_name.clone(), "".to_string()))
        .path(container.cookie_path.clone())
        .http_only(container.cookie_http_only)
        .same_site(same_site)
        .max_age(time::Duration::ZERO);
    
    if container.cookie_secure {
        cookie_builder = cookie_builder.secure(true);
    }
    
    if !container.cookie_domain.is_empty() 
        && container.cookie_domain != "localhost" 
        && container.cookie_domain != "127.0.0.1" 
    {
        cookie_builder = cookie_builder.domain(container.cookie_domain.clone());
    }
    
    cookies.add(cookie_builder.build());
}
