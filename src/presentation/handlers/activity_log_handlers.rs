use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::{info, instrument};

use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::handlers::common::json_ok;
use crate::presentation::extractors::AuthUser;
use crate::application::dtos::{ActivityLogDto, ActivityLogListDto};
use crate::application::ports::ActivityLogFilters;

/// Parámetros de consulta para listar logs
#[derive(Debug, Deserialize)]
pub struct ListLogsParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// Filtrar por tipo de acción (auth, crud, system)
    pub action_type: Option<String>,
    /// Filtrar por acción específica (login, logout, create, update, delete, etc.)
    pub action: Option<String>,
    /// Filtrar por tipo de entidad (user, agencia, tour, etc.)
    pub entity_type: Option<String>,
    /// Filtrar por ID de entidad
    pub entity_id: Option<i32>,
    /// Filtrar por ID de usuario que realizó la acción
    pub user_id: Option<i32>,
    /// Filtrar por estado (success, failed, pending)
    pub status: Option<String>,
    /// Filtrar desde fecha (ISO 8601)
    pub from_date: Option<String>,
    /// Filtrar hasta fecha (ISO 8601)
    pub to_date: Option<String>,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 50 }

/// Listar logs de actividad con filtros y paginación
/// Solo accesible para SuperAdmin
#[instrument(skip(state, auth))]
pub async fn list_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListLogsParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que es SuperAdmin
    if !auth.user.role.is_super_admin() {
        return Err(ApplicationError::Forbidden(
            "Solo SuperAdmin puede ver los logs de actividad".to_string()
        ));
    }
    
    info!("Listando logs de actividad (page: {}, size: {})", params.page, params.page_size);
    
    let page_size = params.page_size.min(100).max(1);
    let offset = (params.page - 1).max(0) * page_size;
    
    // Construir filtros
    let filters = ActivityLogFilters {
        action_type: params.action_type,
        action: params.action,
        entity_type: params.entity_type,
        entity_id: params.entity_id,
        user_id: params.user_id,
        status: params.status,
        from_date: params.from_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&chrono::Utc))),
        to_date: params.to_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&chrono::Utc))),
    };
    
    let logs = state.container.logging_service
        .list_logs(filters.clone(), page_size, offset)
        .await?;
    
    // Obtener total para paginación
    let total = state.container.activity_log_repository
        .count(filters)
        .await?;
    
    let dto_logs: Vec<ActivityLogDto> = logs.into_iter().map(ActivityLogDto::from).collect();
    
    let list_dto = ActivityLogListDto {
        logs: dto_logs.clone(),
        total,
        page: params.page,
        page_size,
        total_pages: (total as f64 / page_size as f64).ceil() as i64,
    };
    
    Ok(json_ok(list_dto))
}

/// Obtener resumen de logs (estadísticas)
#[instrument(skip(state, auth))]
pub async fn get_logs_summary(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que es SuperAdmin
    if !auth.user.role.is_super_admin() {
        return Err(ApplicationError::Forbidden(
            "Solo SuperAdmin puede ver el resumen de logs".to_string()
        ));
    }
    
    info!("Obteniendo resumen de logs");
    
    let summary = state.container.logging_service.get_summary().await?;
    
    Ok(json_ok(summary))
}

/// Obtener errores recientes
#[instrument(skip(state, auth))]
pub async fn get_recent_errors(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(limit): Query<LimitParam>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que es SuperAdmin
    if !auth.user.role.is_super_admin() {
        return Err(ApplicationError::Forbidden(
            "Solo SuperAdmin puede ver los errores recientes".to_string()
        ));
    }
    
    let limit = limit.limit.unwrap_or(20).min(100);
    info!("🔴 Obteniendo {} errores recientes", limit);
    
    let errors = state.container.activity_log_repository
        .find_recent_errors(limit)
        .await?;
    
    let dto_errors: Vec<ActivityLogDto> = errors.into_iter().map(ActivityLogDto::from).collect();
    
    Ok(json_ok(dto_errors))
}

#[derive(Debug, Deserialize)]
pub struct LimitParam {
    pub limit: Option<i64>,
}

/// Limpiar logs antiguos (más de X días)
#[instrument(skip(state, auth))]
pub async fn cleanup_old_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<CleanupParams>,
) -> Result<impl IntoResponse, ApplicationError> {
    // Verificar que es SuperAdmin
    if !auth.user.role.is_super_admin() {
        return Err(ApplicationError::Forbidden(
            "Solo SuperAdmin puede limpiar logs antiguos".to_string()
        ));
    }
    
    let days = params.older_than_days.unwrap_or(90).max(30); // Mínimo 30 días
    info!("🧹 Limpiando logs más antiguos de {} días", days);
    
    let deleted = state.container.activity_log_repository
        .cleanup_old_logs(days)
        .await?;
    
    info!("{} logs eliminados", deleted);
    
    Ok(json_ok(serde_json::json!({
        "deleted_count": deleted,
        "older_than_days": days,
        "message": format!("Se eliminaron {} logs con más de {} días de antigüedad", deleted, days)
    })))
}

#[derive(Debug, Deserialize)]
pub struct CleanupParams {
    pub older_than_days: Option<i64>,
}
