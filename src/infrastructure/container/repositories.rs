//! Creación de repositorios para el contenedor de dependencias

use std::sync::Arc;

use crate::application::ports::{
    UserRepositoryPort, SessionRepositoryPort, PersonaRepositoryPort,
    AgenciaRepositoryPort, TourRepositoryPort, TransporteRepositoryPort,
    VehiculoRepositoryPort, ConductorRepositoryPort, GuiaRepositoryPort,
    RestauranteRepositoryPort, EntradaRepositoryPort, EntradaPrecioRepositoryPort,
    FileRepositoryPort, ActivityLogRepositoryPort,
    NotificationRepositoryPort, FileEntradaRepositoryPort, FileGuiaRepositoryPort,
    FilePasajeroRepositoryPort, FileRestauranteRepositoryPort, FileVehiculoRepositoryPort,
    FileTourRepositoryPort, CachePort,
    // Contabilidad
    PagoFileRepositoryPort,
    PagoProveedorRepositoryPort,
};
use crate::infrastructure::persistence::{
    DatabasePool,
    repositories::{
        PostgresUserRepository, PostgresSessionRepository, PostgresPersonaRepository,
        PostgresAgenciaRepository, PostgresTourRepository, PostgresTransporteRepository,
        PostgresVehiculoRepository, PostgresConductorRepository, PostgresGuiaRepository,
        PostgresRestauranteRepository, PostgresEntradaRepository, PostgresEntradaPrecioRepository,
        PostgresFileRepository, PostgresActivityLogRepository,
        PostgresNotificationRepository, PostgresFileEntradaRepository, PostgresFileGuiaRepository,
        PostgresFilePasajeroRepository, PostgresFileRestauranteRepository,
        PostgresFileVehiculoRepository, PostgresFileTourRepository,
        // Contabilidad
        PostgresPagoFileRepository,
        PostgresPagoProveedorRepository,
    },
};

/// Conjunto de todos los repositorios del sistema
pub(super) struct Repositories {
    pub user: Arc<dyn UserRepositoryPort>,
    pub session: Arc<dyn SessionRepositoryPort>,
    pub persona: Arc<dyn PersonaRepositoryPort>,
    pub agencia: Arc<dyn AgenciaRepositoryPort>,
    pub tour: Arc<dyn TourRepositoryPort>,
    pub transporte: Arc<dyn TransporteRepositoryPort>,
    pub vehiculo: Arc<dyn VehiculoRepositoryPort>,
    pub conductor: Arc<dyn ConductorRepositoryPort>,
    pub guia: Arc<dyn GuiaRepositoryPort>,
    pub restaurante: Arc<dyn RestauranteRepositoryPort>,
    pub entrada: Arc<dyn EntradaRepositoryPort>,
    pub entrada_precio: Arc<dyn EntradaPrecioRepositoryPort>,
    pub file: Arc<dyn FileRepositoryPort>,
    pub activity_log: Arc<dyn ActivityLogRepositoryPort>,
    pub notification: Arc<dyn NotificationRepositoryPort>,
    // File relations
    pub file_entrada: Arc<dyn FileEntradaRepositoryPort>,
    pub file_guia: Arc<dyn FileGuiaRepositoryPort>,
    pub file_pasajero: Arc<dyn FilePasajeroRepositoryPort>,
    pub file_restaurante: Arc<dyn FileRestauranteRepositoryPort>,
    pub file_vehiculo: Arc<dyn FileVehiculoRepositoryPort>,
    pub file_tour: Arc<dyn FileTourRepositoryPort>,
    // Contabilidad
    pub pago_file: Arc<dyn PagoFileRepositoryPort>,
    pub pago_proveedor: Arc<dyn PagoProveedorRepositoryPort>,
}

impl Repositories {
    pub fn create(db_pool: DatabasePool, cache: Arc<dyn CachePort>) -> Self {
        // Repositorios base
        let user = Arc::new(PostgresUserRepository::new(db_pool.clone())) as Arc<dyn UserRepositoryPort>;
        let session = Arc::new(PostgresSessionRepository::new(db_pool.clone())) as Arc<dyn SessionRepositoryPort>;

        // Repositorios de entidades
        let persona = Arc::new(PostgresPersonaRepository::new(db_pool.clone())) as Arc<dyn PersonaRepositoryPort>;
        let agencia = Arc::new(PostgresAgenciaRepository::new(db_pool.clone(), cache.clone())) as Arc<dyn AgenciaRepositoryPort>;
        let tour = Arc::new(PostgresTourRepository::new(db_pool.clone(), cache.clone())) as Arc<dyn TourRepositoryPort>;
        let transporte = Arc::new(PostgresTransporteRepository::new(db_pool.clone())) as Arc<dyn TransporteRepositoryPort>;
        let vehiculo = Arc::new(PostgresVehiculoRepository::new(db_pool.clone())) as Arc<dyn VehiculoRepositoryPort>;
        let conductor = Arc::new(PostgresConductorRepository::new(db_pool.clone())) as Arc<dyn ConductorRepositoryPort>;
        let guia = Arc::new(PostgresGuiaRepository::new(db_pool.clone())) as Arc<dyn GuiaRepositoryPort>;
        let restaurante = Arc::new(PostgresRestauranteRepository::new(db_pool.clone(), cache.clone())) as Arc<dyn RestauranteRepositoryPort>;
        let entrada = Arc::new(PostgresEntradaRepository::new(db_pool.clone(), cache.clone())) as Arc<dyn EntradaRepositoryPort>;
        let entrada_precio = Arc::new(PostgresEntradaPrecioRepository::new(db_pool.clone(), cache.clone())) as Arc<dyn EntradaPrecioRepositoryPort>;
        let file = Arc::new(PostgresFileRepository::new(db_pool.clone())) as Arc<dyn FileRepositoryPort>;

        // Repositorios de sistema
        let activity_log = Arc::new(PostgresActivityLogRepository::new(db_pool.clone())) as Arc<dyn ActivityLogRepositoryPort>;
        let notification = Arc::new(PostgresNotificationRepository::new(db_pool.clone())) as Arc<dyn NotificationRepositoryPort>;

        // File relations
        let file_entrada = Arc::new(PostgresFileEntradaRepository::new(db_pool.clone())) as Arc<dyn FileEntradaRepositoryPort>;
        let file_guia = Arc::new(PostgresFileGuiaRepository::new(db_pool.clone())) as Arc<dyn FileGuiaRepositoryPort>;
        let file_pasajero = Arc::new(PostgresFilePasajeroRepository::new(db_pool.clone())) as Arc<dyn FilePasajeroRepositoryPort>;
        let file_restaurante = Arc::new(PostgresFileRestauranteRepository::new(db_pool.clone())) as Arc<dyn FileRestauranteRepositoryPort>;
        let file_vehiculo = Arc::new(PostgresFileVehiculoRepository::new(db_pool.clone())) as Arc<dyn FileVehiculoRepositoryPort>;
        let file_tour = Arc::new(PostgresFileTourRepository::new(db_pool.clone())) as Arc<dyn FileTourRepositoryPort>;

        // Contabilidad
        let pago_file = Arc::new(PostgresPagoFileRepository::new(db_pool.clone())) as Arc<dyn PagoFileRepositoryPort>;
        let pago_proveedor = Arc::new(PostgresPagoProveedorRepository::new(db_pool.clone())) as Arc<dyn PagoProveedorRepositoryPort>;

        Self {
            user, session, persona, agencia, tour, transporte, vehiculo,
            conductor, guia, restaurante, entrada, entrada_precio, file,
            activity_log, notification,
            file_entrada, file_guia, file_pasajero, file_restaurante,
            file_vehiculo, file_tour,
            pago_file, pago_proveedor,
        }
    }
}
