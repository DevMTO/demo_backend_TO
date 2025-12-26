//! # TypeScript Types Module
//!
//! Este módulo exporta todos los tipos necesarios para generar
//! archivos TypeScript usando ts-rs.
//!
//! ## Organización
//!
//! Los tipos están organizados por dominio:
//! - `auth_types`: Autenticación (login, logout, tokens)
//! - `user_types`: Usuarios y sesiones
//! - `persona_types`: Personas (datos personales)
//! - `agencia_types`: Agencias de viaje
//! - `tour_types`: Tours
//! - `transporte_types`: Transportes, vehículos, conductores
//! - `guia_types`: Guías turísticos
//! - `restaurante_types`: Restaurantes
//! - `entrada_types`: Entradas (pasajeros en tours)
//! - `file_types`: Archivos y documentos
//! - `pago_types`: Pagos y movimientos financieros
//!
//! ## Uso
//!
//! Para generar los tipos TypeScript ejecutar:
//! ```bash
//! cargo test export_ts_types -- --nocapture
//! ```
//!
//! ## Nota sobre warnings de ts-rs
//!
//! Los warnings "failed to parse serde attribute" de ts-rs son esperados
//! ya que ts-rs no soporta `skip_serializing_if`. Estos atributos son
//! para serde (serialización runtime) y no afectan la generación de tipos.

pub mod agencia_types;
pub mod auth_types;
pub mod entrada_types;
pub mod file_types;
pub mod guia_types;
pub mod pago_types;
pub mod persona_types;
pub mod restaurante_types;
pub mod tour_types;
pub mod transporte_types;
pub mod user_types;

// Solo se usan en los tests de exportación

#[cfg(test)]
mod tests {
    //! Test para exportar todos los tipos a TypeScript.
    //!
    //! Ejecutar con: `cargo test export_ts_types -- --nocapture`

    use super::*;

    #[test]
    fn export_ts_types() {
        use ts_rs::TS;

        // Auth types
        LoginRequestTs::export_all().expect("Error exporting LoginRequestTs");
        AuthResponseTs::export_all().expect("Error exporting AuthResponseTs");
        ChangePasswordRequestTs::export_all().expect("Error exporting ChangePasswordRequestTs");
        LogoutRequestTs::export_all().expect("Error exporting LogoutRequestTs");
        SuccessResponseTs::export_all().expect("Error exporting SuccessResponseTs");
        ErrorResponseTs::export_all().expect("Error exporting ErrorResponseTs");

        // User types
        UserRoleTs::export_all().expect("Error exporting UserRoleTs");
        UserInfoTs::export_all().expect("Error exporting UserInfoTs");
        UserDetailTs::export_all().expect("Error exporting UserDetailTs");
        CreateUserRequestTs::export_all().expect("Error exporting CreateUserRequestTs");
        UpdateUserRequestTs::export_all().expect("Error exporting UpdateUserRequestTs");
        UserListResponseTs::export_all().expect("Error exporting UserListResponseTs");
        PaginationParamsTs::export_all().expect("Error exporting PaginationParamsTs");
        SessionInfoTs::export_all().expect("Error exporting SessionInfoTs");
        UserSessionsResponseTs::export_all().expect("Error exporting UserSessionsResponseTs");

        // Persona types
        PersonaTs::export_all().expect("Error exporting PersonaTs");
        CreatePersonaRequestTs::export_all().expect("Error exporting CreatePersonaRequestTs");
        UpdatePersonaRequestTs::export_all().expect("Error exporting UpdatePersonaRequestTs");
        PersonaListResponseTs::export_all().expect("Error exporting PersonaListResponseTs");

        // Agencia types
        AgenciaTs::export_all().expect("Error exporting AgenciaTs");
        CreateAgenciaRequestTs::export_all().expect("Error exporting CreateAgenciaRequestTs");
        UpdateAgenciaRequestTs::export_all().expect("Error exporting UpdateAgenciaRequestTs");
        AgenciaListResponseTs::export_all().expect("Error exporting AgenciaListResponseTs");

        // Tour types
        TourTs::export_all().expect("Error exporting TourTs");
        CreateTourRequestTs::export_all().expect("Error exporting CreateTourRequestTs");
        UpdateTourRequestTs::export_all().expect("Error exporting UpdateTourRequestTs");
        TourListResponseTs::export_all().expect("Error exporting TourListResponseTs");
        TourDetailTs::export_all().expect("Error exporting TourDetailTs");

        // Transporte types
        StatusVehiculoTs::export_all().expect("Error exporting StatusVehiculoTs");
        StatusConductorTs::export_all().expect("Error exporting StatusConductorTs");
        TransporteTs::export_all().expect("Error exporting TransporteTs");
        CreateTransporteRequestTs::export_all().expect("Error exporting CreateTransporteRequestTs");
        UpdateTransporteRequestTs::export_all().expect("Error exporting UpdateTransporteRequestTs");
        TransporteListResponseTs::export_all().expect("Error exporting TransporteListResponseTs");
        VehiculoTs::export_all().expect("Error exporting VehiculoTs");
        CreateVehiculoRequestTs::export_all().expect("Error exporting CreateVehiculoRequestTs");
        UpdateVehiculoRequestTs::export_all().expect("Error exporting UpdateVehiculoRequestTs");
        VehiculoListResponseTs::export_all().expect("Error exporting VehiculoListResponseTs");
        ConductorTs::export_all().expect("Error exporting ConductorTs");
        CreateConductorRequestTs::export_all().expect("Error exporting CreateConductorRequestTs");
        UpdateConductorRequestTs::export_all().expect("Error exporting UpdateConductorRequestTs");
        ConductorListResponseTs::export_all().expect("Error exporting ConductorListResponseTs");
        ConductorDetailTs::export_all().expect("Error exporting ConductorDetailTs");

        // Guia types
        StatusGuiaTs::export_all().expect("Error exporting StatusGuiaTs");
        GuiaTs::export_all().expect("Error exporting GuiaTs");
        CreateGuiaRequestTs::export_all().expect("Error exporting CreateGuiaRequestTs");
        UpdateGuiaRequestTs::export_all().expect("Error exporting UpdateGuiaRequestTs");
        GuiaListResponseTs::export_all().expect("Error exporting GuiaListResponseTs");
        GuiaDetailTs::export_all().expect("Error exporting GuiaDetailTs");

        // Restaurante types
        RestauranteTs::export_all().expect("Error exporting RestauranteTs");
        CreateRestauranteRequestTs::export_all().expect("Error exporting CreateRestauranteRequestTs");
        UpdateRestauranteRequestTs::export_all().expect("Error exporting UpdateRestauranteRequestTs");
        RestauranteListResponseTs::export_all().expect("Error exporting RestauranteListResponseTs");

        // Entrada types
        EntradaTs::export_all().expect("Error exporting EntradaTs");
        CreateEntradaRequestTs::export_all().expect("Error exporting CreateEntradaRequestTs");
        UpdateEntradaRequestTs::export_all().expect("Error exporting UpdateEntradaRequestTs");
        EntradaListResponseTs::export_all().expect("Error exporting EntradaListResponseTs");
        EntradaDetailTs::export_all().expect("Error exporting EntradaDetailTs");
        EntradaStatsTs::export_all().expect("Error exporting EntradaStatsTs");

        // File types
        StatusFileTs::export_all().expect("Error exporting StatusFileTs");
        FileTs::export_all().expect("Error exporting FileTs");
        CreateFileRequestTs::export_all().expect("Error exporting CreateFileRequestTs");
        UpdateFileRequestTs::export_all().expect("Error exporting UpdateFileRequestTs");
        FileListResponseTs::export_all().expect("Error exporting FileListResponseTs");
        FileUploadResponseTs::export_all().expect("Error exporting FileUploadResponseTs");
        FileDownloadUrlTs::export_all().expect("Error exporting FileDownloadUrlTs");

        // Pago types
        TipoMovimientoTs::export_all().expect("Error exporting TipoMovimientoTs");
        PagoTs::export_all().expect("Error exporting PagoTs");
        CreatePagoRequestTs::export_all().expect("Error exporting CreatePagoRequestTs");
        UpdatePagoRequestTs::export_all().expect("Error exporting UpdatePagoRequestTs");
        PagoListResponseTs::export_all().expect("Error exporting PagoListResponseTs");
        PagoDetailTs::export_all().expect("Error exporting PagoDetailTs");
        PagoResumenEntradaTs::export_all().expect("Error exporting PagoResumenEntradaTs");
        PagoResumenTourTs::export_all().expect("Error exporting PagoResumenTourTs");
        PagoFiltersTs::export_all().expect("Error exporting PagoFiltersTs");

        println!("✅ Todos los tipos TypeScript exportados exitosamente!");
    }
}
