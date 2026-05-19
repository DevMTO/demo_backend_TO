//! POST handlers para Saldo a Favor

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use tracing::instrument;

use crate::application::dtos::contabilidad_dto::{
    CancelarFileRequest, CancelarFileTourRequest,
    RegistrarNoShowRequest, NoShowFileTourRequest,
    AutorizarNoShowSaldoRequest, UsarSaldoFavorRequest,
};
use crate::application::dtos::AuditInfo;
use crate::domain::entities::UserRole;
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;

/// Helper: ¿es admin/superadmin?
fn is_admin(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin)
}

/// Helper: ¿puede gestionar contabilidad?
fn can_manage_contabilidad(role: &UserRole) -> bool {
    matches!(role, UserRole::SuperAdmin | UserRole::Admin | UserRole::AgenciasContador | UserRole::Agencias | UserRole::Hoteles | UserRole::HotelesGerente | UserRole::HotelesGerenteCadena)
}

/// Helper: Verificar si el usuario es dueño del file o de la cadena del hotel del file
async fn check_file_ownership(state: &AppState, auth: &AuthUser, file_id: i32) -> Result<(), ApplicationError> {
    if is_admin(&auth.user.role) { return Ok(()); }
    
    let file = state.container.file_repository.find_by_id(file_id).await?
        .ok_or_else(|| ApplicationError::NotFound(format!("File {} no encontrado", file_id)))?;
        
    let user_entidad = auth.user.id_entidad.unwrap_or(0);
    let check_cadena = if auth.user.role == UserRole::HotelesGerenteCadena {
        if let Ok(Some(hotel)) = state.container.hotel_repository.find_by_id(file.id_entidad).await {
            hotel.id_cadena == user_entidad
        } else {
            false
        }
    } else {
        false
    };

    if file.id_entidad != user_entidad && !check_cadena {
        return Err(ApplicationError::Forbidden("No tienes permiso para operar sobre este file".to_string()));
    }
    
    Ok(())
}

/// Helper: Verificar si el usuario es dueño del file tour
async fn check_file_tour_ownership(state: &AppState, auth: &AuthUser, file_tour_id: i32) -> Result<(), ApplicationError> {
    if is_admin(&auth.user.role) { return Ok(()); }
    
    let ft = state.container.file_tour_repository.find_by_id(file_tour_id).await?
        .ok_or_else(|| ApplicationError::NotFound(format!("FileTour {} no encontrado", file_tour_id)))?;
        
    check_file_ownership(state, auth, ft.id_file).await
}

// ============================================================================
// CANCELACIONES
// ============================================================================

/// POST /api/contabilidad/saldos-favor/cancelar-file
/// Cancela un file completo - todo el monto pagado se convierte en saldo a favor
#[instrument(skip(state, auth))]
pub async fn cancelar_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(mut request): Json<CancelarFileRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !can_manage_contabilidad(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para cancelar files".to_string(),
        ));
    }
    
    check_file_ownership(&state, &auth, request.id_file).await?;

    let nota = request.notas.clone();
    let file_id = request.id_file;
    request.notas = None;

    let result = state
        .container
        .saldo_favor_service
        .cancelar_file(request, Some(auth.user.id))
        .await?;

    if let Some(nota_value) = nota {
        let user_info = AuditInfo {
            user_id: auth.user.id,
            username: auth.user.username.clone(),
            is_admin: auth.user.role.is_admin(),
        };
        let _ = state.container.chat_service
            .chat_file(
                file_id,
                Some(nota_value),
                Some(user_info),
            )
            .await;
    }

    Ok(json_ok(result))
}

/// POST /api/contabilidad/saldos-favor/cancelar-tour
/// Cancela un file_tour específico - parte proporcional a saldo a favor
#[instrument(skip(state, auth))]
pub async fn cancelar_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(mut request): Json<CancelarFileTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !can_manage_contabilidad(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para cancelar tours".to_string(),
        ));
    }
    
    check_file_tour_ownership(&state, &auth, request.id_file_tour).await?;

    let nota = request.notas.clone();
    let file_tour_id = request.id_file_tour;
    request.notas = None;

    let result = state
        .container
        .saldo_favor_service
        .cancelar_file_tour(request, Some(auth.user.id))
        .await?;

    if let Some(nota_value) = nota {
        let user_info = AuditInfo {
            user_id: auth.user.id,
            username: auth.user.username.clone(),
            is_admin: auth.user.role.is_admin(),
        };
        let _ = state.container.chat_service
            .chat_file_tour(
                file_tour_id,
                Some(nota_value),
                Some(user_info),
            )
            .await;
    }

    Ok(json_ok(result))
}

// ============================================================================
// NO-SHOWS
// ============================================================================

/// POST /api/contabilidad/saldos-favor/registrar-no-show
/// Registra no-show de un file completo
#[instrument(skip(state, auth))]
pub async fn registrar_no_show(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<RegistrarNoShowRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !can_manage_contabilidad(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para registrar no-shows".to_string(),
        ));
    }
    
    check_file_ownership(&state, &auth, request.id_file).await?;

    let user_info = AuditInfo {
        user_id: auth.user.id,
        username: auth.user.username.clone(),
        is_admin: auth.user.role.is_admin(),
    };
    let result = state
        .container
        .saldo_favor_service
        .registrar_no_show(request, Some(user_info))
        .await?;

    Ok(json_ok(result))
}

/// POST /api/contabilidad/saldos-favor/registrar-no-show-tour
/// Registra no-show de un file_tour específico
#[instrument(skip(state, auth))]
pub async fn registrar_no_show_file_tour(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<NoShowFileTourRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !can_manage_contabilidad(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para registrar no-shows".to_string(),
        ));
    }
    
    check_file_tour_ownership(&state, &auth, request.id_file_tour).await?;

    let user_info = AuditInfo {
        user_id: auth.user.id,
        username: auth.user.username.clone(),
        is_admin: auth.user.role.is_admin(),
    };
    let result = state
        .container
        .saldo_favor_service
        .registrar_no_show_file_tour(request, Some(user_info))
        .await?;

    Ok(json_ok(result))
}

/// POST /api/contabilidad/saldos-favor/autorizar-saldo
/// Autoriza saldo a favor de un no-show (solo admin/superadmin)
#[instrument(skip(state, auth))]
pub async fn autorizar_no_show_saldo(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<AutorizarNoShowSaldoRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !is_admin(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "Solo administradores pueden autorizar saldos de no-show".to_string(),
        ));
    }

    let result = state
        .container
        .saldo_favor_service
        .autorizar_no_show_saldo(request, auth.user.id)
        .await?;

    Ok(json_ok(result))
}

// ============================================================================
// USO DE SALDO
// ============================================================================

/// POST /api/contabilidad/saldos-favor/usar-saldo
/// Aplica saldo a favor al pago de un file
#[instrument(skip(state, auth))]
pub async fn usar_saldo(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<UsarSaldoFavorRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    if !can_manage_contabilidad(&auth.user.role) {
        return Err(ApplicationError::Forbidden(
            "No tienes permiso para aplicar saldos".to_string(),
        ));
    }
    
    check_file_ownership(&state, &auth, request.id_file).await?;
    
    if !is_admin(&auth.user.role) {
        let user_entidad = auth.user.id_entidad.unwrap_or(0);
        let check_cadena = if auth.user.role == UserRole::HotelesGerenteCadena {
            if let Ok(Some(hotel)) = state.container.hotel_repository.find_by_id(request.id_entidad).await {
                hotel.id_cadena == user_entidad
            } else {
                false
            }
        } else {
            false
        };

        if request.id_entidad != user_entidad && !check_cadena {
            return Err(ApplicationError::Forbidden("No tienes permiso para usar saldo de otra entidad".to_string()));
        }
    }

    let result = state
        .container
        .saldo_favor_service
        .usar_saldo(request, Some(auth.user.id))
        .await?;

    Ok(json_ok(result))
}
