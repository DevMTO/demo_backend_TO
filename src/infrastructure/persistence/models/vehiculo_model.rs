use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::entities::{Vehiculo, StatusVehiculo};
use crate::infrastructure::persistence::schema::vehiculos;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = vehiculos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VehiculoModel {
    pub id: i32,
    pub id_transporte: i32,
    pub nombre: String,
    pub modelo: Option<String>,
    pub placa: String,
    pub capacidad: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub is_active: bool,
    pub capacidad_disponible: i32,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = vehiculos)]
pub struct NewVehiculoModel<'a> {
    pub id_transporte: i32,
    pub nombre: &'a str,
    pub modelo: Option<&'a str>,
    pub placa: &'a str,
    pub capacidad: i32,
    pub capacidad_disponible: i32,
    pub status: &'a str,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub is_active: bool,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = vehiculos)]
pub struct UpdateVehiculoModel<'a> {
    pub nombre: Option<&'a str>,
    pub modelo: Option<Option<&'a str>>,
    pub placa: Option<&'a str>,
    pub capacidad: Option<i32>,
    pub capacidad_disponible: Option<i32>,
    pub status: Option<&'a str>,
    pub updated_by: Option<i32>,
    pub is_active: Option<bool>,
}

impl From<VehiculoModel> for Vehiculo {
    fn from(model: VehiculoModel) -> Self {
        Vehiculo {
            id: model.id,
            id_transporte: model.id_transporte,
            nombre: model.nombre,
            modelo: model.modelo,
            placa: model.placa,
            capacidad: model.capacidad,
            capacidad_disponible: model.capacidad_disponible,
            status: model.status.parse().unwrap_or_default(),
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a Vehiculo> for NewVehiculoModel<'a> {
    fn from(v: &'a Vehiculo) -> Self {
        NewVehiculoModel {
            id_transporte: v.id_transporte,
            nombre: &v.nombre,
            modelo: v.modelo.as_deref(),
            placa: &v.placa,
            capacidad: v.capacidad,
            capacidad_disponible: v.capacidad_disponible,
            status: match &v.status {
                StatusVehiculo::Disponible => "disponible",
                StatusVehiculo::EnUso => "en_uso",
                StatusVehiculo::Mantenimiento => "mantenimiento",
                StatusVehiculo::FueraServicio => "fuera_servicio",
            },
            created_by: v.created_by,
            updated_by: v.updated_by,
            is_active: v.is_active,
        }
    }
}
