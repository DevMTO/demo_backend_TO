use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::domain::entities::{Persona, TipoDocumento};

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct PersonaResponse {
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
}

impl From<Persona> for PersonaResponse {
    fn from(p: Persona) -> Self {
        Self {
            id: p.id,
            tipo_documento: p.tipo_documento.to_string(),
            nro_documento: p.nro_documento,
            nombre: p.nombre,
            apellidos: p.apellidos,
            telefono: p.telefono,
            correo: p.correo,
            fecha_nacimiento: p.fecha_nacimiento,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct CreatePersonaRequest {
    #[validate(length(min = 2, max = 30, message = "Tipo documento inválido"))]
    pub tipo_documento: String,

    #[validate(length(
        min = 6,
        max = 20,
        message = "Nro documento debe tener entre 6 y 20 caracteres"
    ))]
    pub nro_documento: String,

    #[validate(length(
        min = 2,
        max = 100,
        message = "Nombre debe tener entre 2 y 100 caracteres"
    ))]
    pub nombre: String,

    #[validate(length(
        min = 2,
        max = 100,
        message = "Apellidos debe tener entre 2 y 100 caracteres"
    ))]
    pub apellidos: String,

    #[validate(length(max = 20, message = "Teléfono muy largo"))]
    pub telefono: Option<String>,

    #[validate(email(message = "Correo inválido"))]
    pub correo: Option<String>,

    pub fecha_nacimiento: Option<NaiveDate>,
}

impl CreatePersonaRequest {
    pub fn into_entity(self, created_by: Option<i32>) -> Persona {
        let now = Utc::now();
        let tipo = self
            .tipo_documento
            .parse::<TipoDocumento>()
            .unwrap_or(TipoDocumento::Dni);
        Persona {
            id: 0,
            tipo_documento: tipo,
            nro_documento: self.nro_documento,
            nombre: self.nombre,
            apellidos: self.apellidos,
            telefono: self.telefono,
            correo: self.correo,
            fecha_nacimiento: self.fecha_nacimiento,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdatePersonaRequest {
    #[validate(length(min = 2, max = 30))]
    pub tipo_documento: Option<String>,

    #[validate(length(min = 6, max = 20))]
    pub nro_documento: Option<String>,

    #[validate(length(min = 2, max = 100))]
    pub nombre: Option<String>,

    #[validate(length(min = 2, max = 100))]
    pub apellidos: Option<String>,

    #[validate(length(max = 20))]
    pub telefono: Option<String>,

    #[validate(email)]
    pub correo: Option<String>,

    pub fecha_nacimiento: Option<NaiveDate>,
}

impl UpdatePersonaRequest {
    pub fn apply_to(self, mut persona: Persona, updated_by: Option<i32>) -> Persona {
        if let Some(tipo) = self.tipo_documento {
            if let Ok(tipo_enum) = tipo.parse::<TipoDocumento>() {
                persona.tipo_documento = tipo_enum;
            }
        }
        if let Some(nro) = self.nro_documento {
            persona.nro_documento = nro;
        }
        if let Some(nombre) = self.nombre {
            persona.nombre = nombre;
        }
        if let Some(apellidos) = self.apellidos {
            persona.apellidos = apellidos;
        }
        if let Some(telefono) = self.telefono {
            persona.telefono = Some(telefono);
        }
        if let Some(correo) = self.correo {
            persona.correo = Some(correo);
        }
        if let Some(fecha) = self.fecha_nacimiento {
            persona.fecha_nacimiento = Some(fecha);
        }
        persona.updated_by = updated_by;
        persona.updated_at = Utc::now();
        persona
    }
}
