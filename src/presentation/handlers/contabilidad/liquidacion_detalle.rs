//! Handler bulk para obtener todos los detalles de liquidación en una sola petición.
//! Reemplaza 100+ requests individuales del frontend (tours, entradas, restaurantes, precios).

use axum::{extract::State, response::IntoResponse, Json};
use futures::future::join_all;
use std::collections::{HashMap, HashSet};
use tracing::instrument;

use crate::application::dtos::{
    LiquidacionDetalleRequest, LiquidacionDetalleResponse,
    LiquidacionTourDetalle, LiquidacionEntradaDetalle,
    LiquidacionRestauranteDetalle, LiquidacionPrecioDetalle,
};
use crate::domain::entities::EntradaPrecio;
use crate::domain::errors::ApplicationError;
use crate::presentation::extractors::AuthUser;
use crate::presentation::handlers::common::json_ok;
use crate::presentation::routes::AppState;

/// POST /api/v1/contabilidad/liquidacion-detalle
/// Obtiene todos los datos necesarios para generar PDF/Excel de liquidación en una sola petición.
#[instrument(skip(state, _auth))]
pub async fn get_liquidacion_detalle(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(request): Json<LiquidacionDetalleRequest>,
) -> Result<impl IntoResponse, ApplicationError> {
    let file_ids = &request.file_ids;
    
    if file_ids.is_empty() {
        return Ok(json_ok(LiquidacionDetalleResponse {
            tours_by_file: HashMap::new(),
            entradas_by_file_tour: HashMap::new(),
            restaurantes_by_file_tour: HashMap::new(),
            precios_by_id: HashMap::new(),
            nro_pasajeros_by_file: HashMap::new(),
        }));
    }

    // Limitar cantidad de files para evitar abuso
    if file_ids.len() > 200 {
        return Err(ApplicationError::Validation("Máximo 200 files por consulta".to_string()));
    }

    // === PASO 1: Obtener tours de todos los files en paralelo ===
    let tour_futures = file_ids.iter().map(|&file_id| {
        let repo = state.container.file_tour_repository.clone();
        async move {
            let tours = repo.find_by_file_with_tour(file_id).await?;
            Ok::<(i32, Vec<_>), ApplicationError>((file_id, tours))
        }
    });
    let tour_results = join_all(tour_futures).await;
    
    let mut tours_by_file: HashMap<i32, Vec<LiquidacionTourDetalle>> = HashMap::new();
    let mut all_file_tour_ids: Vec<i32> = Vec::new();
    
    for result in tour_results {
        let (file_id, tours) = result?;
        let detalles: Vec<LiquidacionTourDetalle> = tours.iter().map(|t| {
            all_file_tour_ids.push(t.id);
            LiquidacionTourDetalle {
                id: t.id,
                id_file: t.id_file,
                id_tour: t.id_tour,
                orden: t.orden,
                precio_aplicado: t.precio_aplicado.as_ref().map(|p| p.to_string()),
                fecha_tour: t.fecha_tour.map(|d| d.to_string()),
                turno_tour: t.turno_tour.clone(),
                status: t.status.clone(),
                nro_pasajeros: t.nro_pasajeros,
                tour_nombre: Some(t.tour_nombre.clone()),
                tour_precio_base: None, // Not needed from here, comes from tarifa
            }
        }).collect();
        tours_by_file.insert(file_id, detalles);
    }

    // === PASO 2: Obtener entradas y restaurantes de todos los tours en paralelo ===
    let entrada_futures = all_file_tour_ids.iter().map(|&ft_id| {
        let repo = state.container.file_entrada_repository.clone();
        let entrada_repo = state.container.entrada_repository.clone();
        async move {
            let entradas = repo.find_by_file_tour(ft_id).await?;
            let mut detalles = Vec::with_capacity(entradas.len());
            for e in &entradas {
                let nombre = if let Ok(Some(entrada)) = entrada_repo.find_by_id(e.id_entrada).await {
                    Some(entrada.nombre)
                } else {
                    None
                };
                detalles.push(LiquidacionEntradaDetalle {
                    id: e.id,
                    id_file_tour: e.id_file_tour,
                    id_entrada: e.id_entrada,
                    cantidad: e.cantidad,
                    id_entrada_precio: e.id_entrada_precio,
                    status: e.status.clone(),
                    entrada_nombre: nombre,
                    entrada_precio: None, // Resolved via precios_by_id
                });
            }
            Ok::<(i32, Vec<LiquidacionEntradaDetalle>), ApplicationError>((ft_id, detalles))
        }
    });

    let restaurante_futures = all_file_tour_ids.iter().map(|&ft_id| {
        let repo = state.container.file_restaurante_repository.clone();
        let rest_repo = state.container.restaurante_repository.clone();
        async move {
            let restaurantes = repo.find_by_file_tour(ft_id).await?;
            let mut detalles = Vec::with_capacity(restaurantes.len());
            for r in &restaurantes {
                let nombre = if let Ok(Some(rest)) = rest_repo.find_by_id(r.id_restaurante).await {
                    Some(rest.nombre)
                } else {
                    None
                };
                detalles.push(LiquidacionRestauranteDetalle {
                    id: r.id,
                    id_file_tour: r.id_file_tour,
                    id_restaurante: r.id_restaurante,
                    precio: r.precio.as_ref().map(|p| p.to_string()),
                    status: r.status.clone(),
                    restaurante_nombre: nombre,
                });
            }
            Ok::<(i32, Vec<LiquidacionRestauranteDetalle>), ApplicationError>((ft_id, detalles))
        }
    });

    // Obtener nro_pasajeros de todos los files en paralelo
    let file_detail_futures = file_ids.iter().map(|&file_id| {
        let repo = state.container.file_repository.clone();
        async move {
            let file = repo.find_by_id(file_id).await?;
            Ok::<(i32, Option<i32>), ApplicationError>((file_id, file.map(|f| f.nro_pasajeros)))
        }
    });

    // Ejecutar todo en paralelo
    let (entrada_results, restaurante_results, file_results) = tokio::join!(
        join_all(entrada_futures),
        join_all(restaurante_futures),
        join_all(file_detail_futures),
    );

    // Procesar entradas
    let mut entradas_by_file_tour: HashMap<i32, Vec<LiquidacionEntradaDetalle>> = HashMap::new();
    let mut precio_ids_needed: HashSet<i32> = HashSet::new();
    
    for result in entrada_results {
        let (ft_id, entradas) = result?;
        for e in &entradas {
            if let Some(precio_id) = e.id_entrada_precio {
                precio_ids_needed.insert(precio_id);
            }
        }
        entradas_by_file_tour.insert(ft_id, entradas);
    }

    // Procesar restaurantes
    let mut restaurantes_by_file_tour: HashMap<i32, Vec<LiquidacionRestauranteDetalle>> = HashMap::new();
    for result in restaurante_results {
        let (ft_id, restaurantes) = result?;
        restaurantes_by_file_tour.insert(ft_id, restaurantes);
    }

    // Procesar nro_pasajeros
    let mut nro_pasajeros_by_file: HashMap<i32, i32> = HashMap::new();
    for result in file_results {
        let (file_id, nro) = result?;
        if let Some(n) = nro {
            nro_pasajeros_by_file.insert(file_id, n);
        }
    }

    // === PASO 3: Obtener precios de entrada en paralelo ===
    let precio_futures = precio_ids_needed.iter().map(|&precio_id| {
        let repo = state.container.entrada_precio_service.clone();
        async move {
            match repo.get_precio(precio_id).await {
                Ok(precio) => Ok(Some((precio_id, precio))),
                Err(_) => Ok::<Option<(i32, EntradaPrecio)>, ApplicationError>(None),
            }
        }
    });
    let precio_results = join_all(precio_futures).await;

    let mut precios_by_id: HashMap<i32, LiquidacionPrecioDetalle> = HashMap::new();
    for result in precio_results {
        if let Some((id, precio)) = result? {
            let rango_label = precio.rango_label();
            precios_by_id.insert(id, LiquidacionPrecioDetalle {
                id: precio.id,
                id_entrada: precio.id_entrada,
                tipo_precio: precio.tipo_precio,
                edad_minima: precio.edad_minima,
                edad_maxima: precio.edad_maxima,
                precio: precio.precio.to_string(),
                rango_label,
            });
        }
    }

    Ok(json_ok(LiquidacionDetalleResponse {
        tours_by_file,
        entradas_by_file_tour,
        restaurantes_by_file_tour,
        precios_by_id,
        nro_pasajeros_by_file,
    }))
}
