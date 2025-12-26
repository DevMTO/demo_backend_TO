//! # Tour Types - TypeScript Exports
//!
//! Tipos de tour exportables a TypeScript.

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Información de tour
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct TourTs {
    pub id: Uuid,
    pub codigo: String,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub fecha_inicio: Option<NaiveDate>,
    pub fecha_fin: Option<NaiveDate>,
    pub hora_inicio: Option<NaiveTime>,
    pub hora_fin: Option<NaiveTime>,
    pub punto_partida: Option<String>,
    pub punto_llegada: Option<String>,
    pub capacidad_maxima: Option<i32>,
    pub precio_base: Option<f64>,
    pub is_active: bool,
    // Referencias
    pub id_agencia: Option<Uuid>,
    pub id_transporte: Option<Uuid>,
    pub id_guia: Option<Uuid>,
    pub id_restaurante: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request para crear tour
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct CreateTourRequestTs {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub fecha_inicio: Option<NaiveDate>,
    pub fecha_fin: Option<NaiveDate>,
    pub hora_inicio: Option<NaiveTime>,
    pub hora_fin: Option<NaiveTime>,
    pub punto_partida: Option<String>,
    pub punto_llegada: Option<String>,
    pub capacidad_maxima: Option<i32>,
    pub precio_base: Option<f64>,
    // Referencias
    pub id_agencia: Option<Uuid>,
    pub id_transporte: Option<Uuid>,
    pub id_guia: Option<Uuid>,
    pub id_restaurante: Option<Uuid>,
}

/// Request para actualizar tour
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct UpdateTourRequestTs {
    pub nombre: Option<String>,
    pub descripcion: Option<String>,
    pub fecha_inicio: Option<NaiveDate>,
    pub fecha_fin: Option<NaiveDate>,
    pub hora_inicio: Option<NaiveTime>,
    pub hora_fin: Option<NaiveTime>,
    pub punto_partida: Option<String>,
    pub punto_llegada: Option<String>,
    pub capacidad_maxima: Option<i32>,
    pub precio_base: Option<f64>,
    pub is_active: Option<bool>,
    pub id_agencia: Option<Uuid>,
    pub id_transporte: Option<Uuid>,
    pub id_guia: Option<Uuid>,
    pub id_restaurante: Option<Uuid>,
}

/// Lista paginada de tours
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct TourListResponseTs {
    pub tours: Vec<TourTs>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

/// Tour con detalles expandidos
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
#[serde(rename_all = "camelCase")]
pub struct TourDetailTs {
    pub tour: TourTs,
    pub agencia_nombre: Option<String>,
    pub transporte_placa: Option<String>,
    pub guia_nombre: Option<String>,
    pub restaurante_nombre: Option<String>,
    pub entradas_count: i64,
    pub monto_total_esperado: f64,
    pub monto_total_pagado: f64,
}
