use std::sync::Arc;

use crate::config::AppConfig;
use crate::application::ports::{
    UserRepositoryPort,
    SessionRepositoryPort,
    PasswordHasherPort,
    SessionManagerPort,
    PersonaRepositoryPort,
    AgenciaRepositoryPort,
    TourRepositoryPort,
    TransporteRepositoryPort,
    VehiculoRepositoryPort,
    ConductorRepositoryPort,
    GuiaRepositoryPort,
    RestauranteRepositoryPort,
    EntradaRepositoryPort,
    EntradaPrecioRepositoryPort,
    FileRepositoryPort,
    PagoRepositoryPort,
    ActivityLogRepositoryPort,
    NotificationRepositoryPort,
    FileEntradaRepositoryPort,
    FileGuiaRepositoryPort,
    FilePasajeroRepositoryPort,
    FileRestauranteRepositoryPort,
    FileVehiculoRepositoryPort,
    FileTourRepositoryPort,
    // Contabilidad ports
    CuentaRepositoryPort,
    MovimientoRepositoryPort,
    PagoFileRepositoryPort,
    PagoProveedorRepositoryPort,
    TarifaServicioRepositoryPort,
};
use crate::application::services::{
    LoggingService,
    NotificationService,
    UserService,
    AgenciaService,
    PersonaService,
    TourService,
    FileService,
    PagoService,
    RestauranteService,
    TransporteService,
    VehiculoService,
    ConductorService,
    EntradaService,
    EntradaPrecioService,
    GuiaService,
    MyFilesService,
    PostgresMyFilesRepository,
    ContabilidadService,
};
use crate::application::use_cases::auth::{
    LoginUseCase,
    LogoutUseCase,
    VerifySessionUseCase,
};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::{
    DatabasePool,
    repositories::{
        PostgresUserRepository, 
        PostgresSessionRepository,
        PostgresPersonaRepository,
        PostgresAgenciaRepository,
        PostgresTourRepository,
        PostgresTransporteRepository,
        PostgresVehiculoRepository,
        PostgresConductorRepository,
        PostgresGuiaRepository,
        PostgresRestauranteRepository,
        PostgresEntradaRepository,
        PostgresEntradaPrecioRepository,
        PostgresFileRepository,
        PostgresPagoRepository,
        PostgresActivityLogRepository,
        PostgresNotificationRepository,
        PostgresFileEntradaRepository,
        PostgresFileGuiaRepository,
        PostgresFilePasajeroRepository,
        PostgresFileRestauranteRepository,
        PostgresFileVehiculoRepository,
        PostgresFileTourRepository,
        // Contabilidad repositories
        PostgresCuentaRepository,
        PostgresMovimientoRepository,
        PostgresPagoFileRepository,
        PostgresPagoProveedorRepository,
        PostgresTarifaServicioRepository,
    },
};
use crate::infrastructure::security::{
    Argon2PasswordHasher,
    SecureSessionManager,
};

#[allow(dead_code)]
pub struct DependencyContainer {
    // Use Cases - Auth
    pub login_use_case: Arc<LoginUseCase>,
    pub logout_use_case: Arc<LogoutUseCase>,
    pub verify_session_use_case: Arc<VerifySessionUseCase>,
    
    // Session Manager para middleware
    pub session_manager: Arc<dyn SessionManagerPort>,
    
    // Password Hasher para crear/actualizar usuarios
    pub password_hasher: Arc<dyn PasswordHasherPort>,
    
    // System Services (Logging & Notifications)
    pub logging_service: Arc<LoggingService>,
    pub notification_service: Arc<NotificationService>,
    
    // Business Services
    pub user_service: Arc<UserService>,
    pub agencia_service: Arc<AgenciaService>,
    pub persona_service: Arc<PersonaService>,
    pub tour_service: Arc<TourService>,
    pub file_service: Arc<FileService>,
    pub pago_service: Arc<PagoService>,
    pub restaurante_service: Arc<RestauranteService>,
    pub transporte_service: Arc<TransporteService>,
    pub vehiculo_service: Arc<VehiculoService>,
    pub conductor_service: Arc<ConductorService>,
    pub entrada_service: Arc<EntradaService>,
    pub entrada_precio_service: Arc<EntradaPrecioService>,
    pub guia_service: Arc<GuiaService>,
    pub my_files_service: Arc<MyFilesService>,
    pub contabilidad_service: Arc<ContabilidadService>,
    
    // Object Storage (Tigris) - Opcional, puede ser None si no está configurado
    pub tigris_storage: Option<Arc<crate::infrastructure::storage::TigrisStorage>>,
    
    // Entity Repositories (para operaciones simples que no necesitan use case)
    pub user_repository: Arc<dyn UserRepositoryPort>,
    pub persona_repository: Arc<dyn PersonaRepositoryPort>,
    pub agencia_repository: Arc<dyn AgenciaRepositoryPort>,
    pub tour_repository: Arc<dyn TourRepositoryPort>,
    pub transporte_repository: Arc<dyn TransporteRepositoryPort>,
    pub vehiculo_repository: Arc<dyn VehiculoRepositoryPort>,
    pub conductor_repository: Arc<dyn ConductorRepositoryPort>,
    pub guia_repository: Arc<dyn GuiaRepositoryPort>,
    pub restaurante_repository: Arc<dyn RestauranteRepositoryPort>,
    pub entrada_repository: Arc<dyn EntradaRepositoryPort>,
    pub file_repository: Arc<dyn FileRepositoryPort>,
    pub pago_repository: Arc<dyn PagoRepositoryPort>,
    pub activity_log_repository: Arc<dyn ActivityLogRepositoryPort>,
    pub notification_repository: Arc<dyn NotificationRepositoryPort>,
    
    // File Relations Repositories
    pub file_entrada_repository: Arc<dyn FileEntradaRepositoryPort>,
    pub file_guia_repository: Arc<dyn FileGuiaRepositoryPort>,
    pub file_pasajero_repository: Arc<dyn FilePasajeroRepositoryPort>,
    pub file_restaurante_repository: Arc<dyn FileRestauranteRepositoryPort>,
    pub file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort>,
    pub file_tour_repository: Arc<dyn FileTourRepositoryPort>,
    
    // Config
    pub cookie_name: String,
    pub cookie_secure: bool,
    pub cookie_same_site: String,
    pub cookie_domain: String,
    pub cookie_path: String,
    pub cookie_http_only: bool,
    pub cookie_max_age_hours: i64,
}
use crate::application::ports::NotificationServicePort;
use crate::infrastructure::NotificationBroadcastAdapter;
use crate::infrastructure::sse::NotificationBroadcaster;

impl DependencyContainer {
    pub fn new(
        db_pool: DatabasePool, 
        config: AppConfig,
        broadcaster: Arc<NotificationBroadcaster>,
    ) -> Result<Self, ApplicationError> {
        // Validar configuración de seguridad
        config.validate_security()
            .map_err(|e| ApplicationError::Configuration(e.to_string()))?;
        
        // Crear repositorios base
        let user_repository: Arc<dyn UserRepositoryPort> = Arc::new(
            PostgresUserRepository::new(db_pool.clone())
        );
        let session_repository: Arc<dyn SessionRepositoryPort> = Arc::new(
            PostgresSessionRepository::new(db_pool.clone())
        );
        
        // Crear servicios de seguridad
        let password_hasher: Arc<dyn PasswordHasherPort> = Arc::new(
            Argon2PasswordHasher::with_params(
                config.argon2_memory_size,
                config.argon2_iterations,
                config.argon2_parallelism,
            )?
        );
        
        let session_manager: Arc<dyn SessionManagerPort> = Arc::new(
            SecureSessionManager::new(&config)?
        );
        
        // Crear repositorios de entidades
        let persona_repository: Arc<dyn PersonaRepositoryPort> = Arc::new(
            PostgresPersonaRepository::new(db_pool.clone())
        );
        let agencia_repository: Arc<dyn AgenciaRepositoryPort> = Arc::new(
            PostgresAgenciaRepository::new(db_pool.clone())
        );
        let tour_repository: Arc<dyn TourRepositoryPort> = Arc::new(
            PostgresTourRepository::new(db_pool.clone())
        );
        let transporte_repository: Arc<dyn TransporteRepositoryPort> = Arc::new(
            PostgresTransporteRepository::new(db_pool.clone())
        );
        let vehiculo_repository: Arc<dyn VehiculoRepositoryPort> = Arc::new(
            PostgresVehiculoRepository::new(db_pool.clone())
        );
        let conductor_repository: Arc<dyn ConductorRepositoryPort> = Arc::new(
            PostgresConductorRepository::new(db_pool.clone())
        );
        let guia_repository: Arc<dyn GuiaRepositoryPort> = Arc::new(
            PostgresGuiaRepository::new(db_pool.clone())
        );
        let restaurante_repository: Arc<dyn RestauranteRepositoryPort> = Arc::new(
            PostgresRestauranteRepository::new(db_pool.clone())
        );
        let entrada_repository: Arc<dyn EntradaRepositoryPort> = Arc::new(
            PostgresEntradaRepository::new(db_pool.clone())
        );
        let entrada_precio_repository: Arc<dyn EntradaPrecioRepositoryPort> = Arc::new(
            PostgresEntradaPrecioRepository::new(db_pool.clone())
        );
        let file_repository: Arc<dyn FileRepositoryPort> = Arc::new(
            PostgresFileRepository::new(db_pool.clone())
        );
        let pago_repository: Arc<dyn PagoRepositoryPort> = Arc::new(
            PostgresPagoRepository::new(db_pool.clone())
        );
        
        // Crear repositorios de sistema (logging y notificaciones)
        let activity_log_repository: Arc<dyn ActivityLogRepositoryPort> = Arc::new(
            PostgresActivityLogRepository::new(db_pool.clone())
        );
        let notification_repository: Arc<dyn NotificationRepositoryPort> = Arc::new(
            PostgresNotificationRepository::new(db_pool.clone())
        );
        
        // Crear repositorios de file relations
        let file_entrada_repository: Arc<dyn FileEntradaRepositoryPort> = Arc::new(
            PostgresFileEntradaRepository::new(db_pool.clone())
        );
        let file_guia_repository: Arc<dyn FileGuiaRepositoryPort> = Arc::new(
            PostgresFileGuiaRepository::new(db_pool.clone())
        );
        let file_pasajero_repository: Arc<dyn FilePasajeroRepositoryPort> = Arc::new(
            PostgresFilePasajeroRepository::new(db_pool.clone())
        );
        let file_restaurante_repository: Arc<dyn FileRestauranteRepositoryPort> = Arc::new(
            PostgresFileRestauranteRepository::new(db_pool.clone())
        );
        let file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort> = Arc::new(
            PostgresFileVehiculoRepository::new(db_pool.clone())
        );
        let file_tour_repository: Arc<dyn FileTourRepositoryPort> = Arc::new(
            PostgresFileTourRepository::new(db_pool.clone())
        );
        
        // Crear servicios de sistema
        let logging_service = Arc::new(LoggingService::new(
            activity_log_repository.clone()
        ));
        let notification_service = Arc::new(NotificationService::new(
            notification_repository.clone()
        ));
        
        // Crear adaptador de notificaciones con broadcast SSE
        let notification_broadcast_adapter: Arc<dyn NotificationServicePort> = Arc::new(
            NotificationBroadcastAdapter::new(
                notification_service.clone(),
                notification_repository.clone(),
                broadcaster,
            )
        );
        
        // Crear servicios de negocio
        let user_service = Arc::new(UserService::new(
            user_repository.clone(),
            persona_repository.clone(),
            password_hasher.clone(),
            logging_service.clone(),
            notification_broadcast_adapter.clone(),
        ));
        
        let agencia_service = Arc::new(AgenciaService::new(
            agencia_repository.clone(),
            logging_service.clone(),
            notification_broadcast_adapter.clone(),
        ));
        
        let persona_service = Arc::new(PersonaService::new(
            persona_repository.clone(),
            logging_service.clone(),
        ));
        
        // ========== Crear casos de uso - Auth ==========
        let login_use_case = Arc::new(LoginUseCase::new(
            user_repository.clone(),
            session_repository.clone(),
            password_hasher.clone(),
            session_manager.clone(),
        ));
        
        let logout_use_case = Arc::new(LogoutUseCase::new(
            session_repository.clone(),
        ));
        
        let verify_session_use_case = Arc::new(VerifySessionUseCase::new(
            user_repository.clone(),
            session_repository,
            session_manager.clone(),
        ));
        
        // ========== Crear servicio - Tour ==========
        let tour_service = Arc::new(TourService::new(
            tour_repository.clone(),
            logging_service.clone(),
            notification_broadcast_adapter.clone(),
        ));
        
        // ========== Crear servicio - File ==========
        let file_service = Arc::new(FileService::new(
            file_repository.clone(),
            file_tour_repository.clone(),
            logging_service.clone(),
            notification_broadcast_adapter.clone(),
        ));
        
        // ========== Crear servicio - Pago ==========
        let pago_service = Arc::new(PagoService::new(
            pago_repository.clone(),
            file_repository.clone(),
            logging_service.clone(),
            notification_broadcast_adapter.clone(),
        ));
        
        // ========== Crear servicio - Restaurante ==========
        let restaurante_service = Arc::new(RestauranteService::new(
            restaurante_repository.clone(),
            logging_service.clone(),
            notification_broadcast_adapter.clone(),
        ));
        
        // ========== Crear servicio - Transporte ==========
        let transporte_service = Arc::new(TransporteService::new(
            transporte_repository.clone(),
            logging_service.clone(),
            notification_broadcast_adapter.clone(),
        ));
        
        // ========== Crear servicio - Vehiculo ==========
        let vehiculo_service = Arc::new(VehiculoService::new(
            vehiculo_repository.clone(),
            logging_service.clone(),
            notification_broadcast_adapter.clone(),
        ));
        
        // ========== Crear servicio - Conductor ==========
        let conductor_service = Arc::new(ConductorService::new(
            conductor_repository.clone(),
            logging_service.clone(),
            notification_broadcast_adapter.clone(),
        ));
        
        // ========== Crear servicio - Entrada ==========
        let entrada_service = Arc::new(EntradaService::new(
            entrada_repository.clone(),
            logging_service.clone(),
            notification_broadcast_adapter.clone(),
        ));
        
        // ========== Crear servicio - EntradaPrecio ==========
        let entrada_precio_service = Arc::new(EntradaPrecioService::new(
            entrada_precio_repository,
        ));
        
        // ========== Crear servicio - Guia ==========
        let guia_service = Arc::new(GuiaService::new(
            guia_repository.clone(),
            logging_service.clone(),
            notification_broadcast_adapter,
        ));
        
        // ========== Crear servicio - MyFiles ==========
        let my_files_repository = Arc::new(PostgresMyFilesRepository::new(db_pool.clone()));
        let my_files_service = Arc::new(MyFilesService::new(my_files_repository));
        
        // ========== Crear repositorios y servicio - Contabilidad ==========
        let cuenta_repository: Arc<dyn CuentaRepositoryPort> = Arc::new(
            PostgresCuentaRepository::new(db_pool.clone())
        );
        let movimiento_repository: Arc<dyn MovimientoRepositoryPort> = Arc::new(
            PostgresMovimientoRepository::new(db_pool.clone())
        );
        let pago_file_repository: Arc<dyn PagoFileRepositoryPort> = Arc::new(
            PostgresPagoFileRepository::new(db_pool.clone())
        );
        let pago_proveedor_repository: Arc<dyn PagoProveedorRepositoryPort> = Arc::new(
            PostgresPagoProveedorRepository::new(db_pool.clone())
        );
        let tarifa_repository: Arc<dyn TarifaServicioRepositoryPort> = Arc::new(
            PostgresTarifaServicioRepository::new(db_pool.clone())
        );
        
        let contabilidad_service = Arc::new(ContabilidadService::new(
            cuenta_repository,
            movimiento_repository,
            pago_file_repository,
            pago_proveedor_repository,
            tarifa_repository,
            agencia_repository.clone(),
            file_repository.clone(),
        ));
        
        Ok(Self {
            // Auth use cases
            login_use_case,
            logout_use_case,
            verify_session_use_case,
            // Services
            session_manager,
            password_hasher,
            logging_service,
            notification_service,
            user_service,
            agencia_service,
            persona_service,
            tour_service,
            file_service,
            pago_service,
            restaurante_service,
            transporte_service,
            vehiculo_service,
            conductor_service,
            entrada_service,
            entrada_precio_service,
            guia_service,
            my_files_service,
            contabilidad_service,
            // Repositories
            user_repository,
            persona_repository,
            agencia_repository,
            tour_repository,
            transporte_repository,
            vehiculo_repository,
            conductor_repository,
            guia_repository,
            restaurante_repository,
            entrada_repository,
            file_repository,
            pago_repository,
            activity_log_repository,
            notification_repository,
            // File Relations Repositories
            file_entrada_repository,
            file_guia_repository,
            file_pasajero_repository,
            file_restaurante_repository,
            file_vehiculo_repository,
            file_tour_repository,
            // Cookie config
            cookie_name: config.cookie_name,
            cookie_secure: config.cookie_secure,
            cookie_same_site: config.cookie_same_site,
            cookie_domain: config.cookie_domain,
            cookie_path: config.cookie_path,
            cookie_http_only: config.cookie_http_only,
            cookie_max_age_hours: config.cookie_max_age_hours,
            // Storage se inicializa después (async)
            tigris_storage: None,
        })
    }
    
    /// Inicializa el storage de Tigris (async)
    /// Llamar después de crear el contenedor
    pub async fn init_storage(&mut self) {
        use crate::infrastructure::storage::{TigrisConfig, TigrisStorage};
        use tracing::{info, warn};
        
        match TigrisConfig::from_env() {
            Ok(config) => {
                match TigrisStorage::new(config).await {
                    Ok(storage) => {
                        self.tigris_storage = Some(Arc::new(storage));
                        info!("Tigris Storage inicializado correctamente");
                    }
                    Err(e) => {
                        warn!("No se pudo inicializar Tigris Storage: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Tigris Storage no configurado: {}", e);
            }
        }
    }
}