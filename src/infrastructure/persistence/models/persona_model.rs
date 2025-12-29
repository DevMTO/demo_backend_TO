use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::entities::{Persona, TipoDocumento};
use crate::infrastructure::persistence::schema::personas;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = personas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PersonaModel {
    pub id: i32,
    pub tipo_documento: String,
    pub nro_documento: String,
    pub nombre: String,
    pub apellidos: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub fecha_nacimiento: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = personas)]
pub struct NewPersonaModel<'a> {
    pub tipo_documento: &'a str,
    pub nro_documento: &'a str,
    pub nombre: &'a str,
    pub apellidos: &'a str,
    pub telefono: Option<&'a str>,
    pub correo: Option<&'a str>,
    pub fecha_nacimiento: Option<NaiveDate>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = personas)]
pub struct UpdatePersonaModel<'a> {
    pub tipo_documento: Option<&'a str>,
    pub nro_documento: Option<&'a str>,
    pub nombre: Option<&'a str>,
    pub apellidos: Option<&'a str>,
    pub telefono: Option<Option<&'a str>>,
    pub correo: Option<Option<&'a str>>,
    pub fecha_nacimiento: Option<Option<NaiveDate>>,
    pub updated_by: Option<i32>,
}

impl From<PersonaModel> for Persona {
    fn from(model: PersonaModel) -> Self {
        Persona {
            id: model.id,
            tipo_documento: model.tipo_documento.parse().unwrap_or_default(),
            nro_documento: model.nro_documento,
            nombre: model.nombre,
            apellidos: model.apellidos,
            telefono: model.telefono,
            correo: model.correo,
            fecha_nacimiento: model.fecha_nacimiento,
            created_at: model.created_at,
            updated_at: model.updated_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl<'a> From<&'a Persona> for NewPersonaModel<'a> {
    fn from(persona: &'a Persona) -> Self {
        NewPersonaModel {
            tipo_documento: match &persona.tipo_documento {
                TipoDocumento::Dni => "DNI",
                TipoDocumento::Pasaporte => "PASAPORTE",
                TipoDocumento::CarnetExtranjeria => "CARNET_EXTRANJERIA",
                TipoDocumento::Ruc => "RUC",
                TipoDocumento::Otro => "OTRO",
            },
            nro_documento: &persona.nro_documento,
            nombre: &persona.nombre,
            apellidos: &persona.apellidos,
            telefono: persona.telefono.as_deref(),
            correo: persona.correo.as_deref(),
            fecha_nacimiento: persona.fecha_nacimiento,
            created_by: persona.created_by,
            updated_by: persona.updated_by,
        }
    }
}
