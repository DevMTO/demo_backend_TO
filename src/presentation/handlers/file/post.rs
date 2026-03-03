//! POST handlers para File

use axum::{extract::State, response::IntoResponse, Json};
use bigdecimal::BigDecimal;
use tracing::{info, instrument, warn};
use validator::Validate;

use crate::application::dtos::{CreateFileRequest, ConfirmReservaRequest};
use crate::domain::errors::ApplicationError;
use crate::presentation::routes::AppState;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::{json_created, json_ok};

/// Crear nuevo file
/// 
/// Si el usuario tiene rol "agencias", se auto-asigna su agencia (id_entidad).
/// Si el usuario es superadmin/admin, debe proporcionar id_agencia en el request.
#[instrument(skip(state, auth, request))]
pub async fn create_file(
    State(state): State<AppState>, 
    auth: AuthUser, 
    Json(request): Json<CreateFileRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;
    
    let response = state.container.file_service
        .create_file(
            request, 
            auth.user.id,
            Some(auth.user.username.clone()),
            auth.user.role.clone(),
            auth.user.id_entidad,
        )
        .await?;
    
    Ok(json_created(response))
}

/// Confirmar una reserva (file)
/// 
/// Este endpoint:
/// 1. Cambia el status del file de "reservado" a "confirmado"
/// 2. Crea un registro de pago pendiente (pagos_files)
/// 3. Notifica a los admins
/// 4. Notifica al contador de la agencia (si existe)
/// 5. Registra en el log de actividad
/// 
/// Solo puede ser usado por usuarios con rol de agencia o admin.
#[instrument(skip(state, auth, request))]
pub async fn confirmar_reserva(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<ConfirmReservaRequest>
) -> Result<impl IntoResponse, ApplicationError> {
    request.validate().map_err(|e| ApplicationError::Validation(e.to_string()))?;

    let file_id = request.file_id;
    info!("📋 Confirmando reserva - File ID: {} por usuario: {}", file_id, auth.user.username);

    let response = state.container.file_service
        .confirmar_reserva(
            request,
            auth.user.id,
            Some(auth.user.username.clone()),
        )
        .await?;

    // ===== AUTO-CREAR PAGOS PROVEEDOR (entradas) =====
    // Recorrer los file_tours y crear un pago_proveedor por cada file_entrada
    let tours = state.container.file_tour_repository
        .find_by_file_with_tour(file_id)
        .await
        .unwrap_or_default();

    for ft in &tours {
        let entradas = state.container.file_entrada_repository
            .find_by_file_tour(ft.id)
            .await
            .unwrap_or_default();

        for fe in &entradas {
            // Calcular monto: precio * cantidad
            let monto = if let Some(precio_id) = fe.id_entrada_precio {
                match state.container.entrada_precio_service.get_precio(precio_id).await {
                    Ok(precio) => Some(precio.precio * BigDecimal::from(fe.cantidad)),
                    Err(_) => None,
                }
            } else {
                None
            };

            if let Err(e) = state.container.contabilidad_service
                .auto_create_pago_proveedor(
                    "entrada",
                    None,
                    None,
                    None,
                    Some(fe.id_entrada),
                    Some(ft.id),
                    None,
                    None,
                    None,
                    Some(fe.id),
                    monto,
                    Some(auth.user.id),
                ).await
            {
                warn!("Error al auto-crear pago proveedor para entrada {}: {}", fe.id_entrada, e);
            }
        }
    }

    Ok(json_ok(response))
}
