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
    FileRepositoryPort,
    PagoRepositoryPort,
    ActivityLogRepositoryPort,
    NotificationRepositoryPort,
};
use crate::infrastructure::persistence::repositories::{
    FileEntradaRepositoryPort,
    FileGuiaRepositoryPort,
    FilePasajeroRepositoryPort,
    FileRestauranteRepositoryPort,
    FileVehiculoRepositoryPort,
};
use crate::application::services::{
    LoggingService,
    NotificationService,
};
use crate::application::use_cases::auth::{
    LoginUseCase,
    LogoutUseCase,
    VerifySessionUseCase,
};
use crate::application::use_cases::persona::{
    CreatePersonaUseCase,
    UpdatePersonaUseCase,
    SearchPersonasUseCase,
};
use crate::application::use_cases::agencia::{
    CreateAgenciaUseCase,
    UpdateAgenciaUseCase,
    DeactivateAgenciaUseCase,
    RestoreAgenciaUseCase,
};
use crate::application::use_cases::tour::{
    CreateTourUseCase,
    UpdateTourUseCase,
    SearchToursUseCase,
    DeactivateTourUseCase,
    RestoreTourUseCase,
};
use crate::application::use_cases::file::{
    CreateFileUseCase,
    UpdateFileUseCase,
    SearchFilesUseCase,
};
use crate::application::use_cases::pago::{
    RegisterPagoUseCase,
    UpdatePagoUseCase,
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
        PostgresFileRepository,
        PostgresPagoRepository,
        PostgresActivityLogRepository,
        PostgresNotificationRepository,
        PostgresFileEntradaRepository,
        PostgresFileGuiaRepository,
        PostgresFilePasajeroRepository,
        PostgresFileRestauranteRepository,
        PostgresFileVehiculoRepository,
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
    
    // Use Cases - Persona
    pub create_persona_use_case: Arc<CreatePersonaUseCase>,
    pub update_persona_use_case: Arc<UpdatePersonaUseCase>,
    pub search_personas_use_case: Arc<SearchPersonasUseCase>,
    
    // Use Cases - Agencia
    pub create_agencia_use_case: Arc<CreateAgenciaUseCase>,
    pub update_agencia_use_case: Arc<UpdateAgenciaUseCase>,
    pub deactivate_agencia_use_case: Arc<DeactivateAgenciaUseCase>,
    pub restore_agencia_use_case: Arc<RestoreAgenciaUseCase>,
    
    // Use Cases - Tour
    pub create_tour_use_case: Arc<CreateTourUseCase>,
    pub update_tour_use_case: Arc<UpdateTourUseCase>,
    pub search_tours_use_case: Arc<SearchToursUseCase>,
    pub deactivate_tour_use_case: Arc<DeactivateTourUseCase>,
    pub restore_tour_use_case: Arc<RestoreTourUseCase>,
    
    // Use Cases - File
    pub create_file_use_case: Arc<CreateFileUseCase>,
    pub update_file_use_case: Arc<UpdateFileUseCase>,
    pub search_files_use_case: Arc<SearchFilesUseCase>,
    
    // Use Cases - Pago
    pub register_pago_use_case: Arc<RegisterPagoUseCase>,
    pub update_pago_use_case: Arc<UpdatePagoUseCase>,
    
    // Session Manager para middleware
    pub session_manager: Arc<dyn SessionManagerPort>,
    
    // Password Hasher para crear/actualizar usuarios
    pub password_hasher: Arc<dyn PasswordHasherPort>,
    
    // System Services (Logging & Notifications)
    pub logging_service: Arc<LoggingService>,
    pub notification_service: Arc<NotificationService>,
    
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
    
    // Config
    pub cookie_name: String,
    pub cookie_secure: bool,
    pub cookie_same_site: String,
    pub cookie_domain: String,
    pub cookie_path: String,
    pub cookie_http_only: bool,
    pub cookie_max_age_hours: i64,
}

impl DependencyContainer {
    pub fn new(db_pool: DatabasePool, config: AppConfig) -> Result<Self, ApplicationError> {
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
            PostgresFileVehiculoRepository::new(db_pool)
        );
        
        // Crear servicios de sistema
        let logging_service = Arc::new(LoggingService::new(
            activity_log_repository.clone()
        ));
        let notification_service = Arc::new(NotificationService::new(
            notification_repository.clone()
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
        
        // ========== Crear casos de uso - Persona ==========
        let create_persona_use_case = Arc::new(CreatePersonaUseCase::new(
            persona_repository.clone()
        ));
        let update_persona_use_case = Arc::new(UpdatePersonaUseCase::new(
            persona_repository.clone()
        ));
        let search_personas_use_case = Arc::new(SearchPersonasUseCase::new(
            persona_repository.clone()
        ));
        
        // ========== Crear casos de uso - Agencia ==========
        let create_agencia_use_case = Arc::new(CreateAgenciaUseCase::new(
            agencia_repository.clone()
        ));
        let update_agencia_use_case = Arc::new(UpdateAgenciaUseCase::new(
            agencia_repository.clone()
        ));
        let deactivate_agencia_use_case = Arc::new(DeactivateAgenciaUseCase::new(
            agencia_repository.clone()
        ));
        let restore_agencia_use_case = Arc::new(RestoreAgenciaUseCase::new(
            agencia_repository.clone()
        ));
        
        // ========== Crear casos de uso - Tour ==========
        let create_tour_use_case = Arc::new(CreateTourUseCase::new(
            tour_repository.clone()
        ));
        let update_tour_use_case = Arc::new(UpdateTourUseCase::new(
            tour_repository.clone()
        ));
        let search_tours_use_case = Arc::new(SearchToursUseCase::new(
            tour_repository.clone()
        ));
        let deactivate_tour_use_case = Arc::new(DeactivateTourUseCase::new(
            tour_repository.clone()
        ));
        let restore_tour_use_case = Arc::new(RestoreTourUseCase::new(
            tour_repository.clone()
        ));
        
        // ========== Crear casos de uso - File ==========
        let create_file_use_case = Arc::new(CreateFileUseCase::new(
            file_repository.clone()
        ));
        let update_file_use_case = Arc::new(UpdateFileUseCase::new(
            file_repository.clone()
        ));
        let search_files_use_case = Arc::new(SearchFilesUseCase::new(
            file_repository.clone()
        ));
        
        // ========== Crear casos de uso - Pago ==========
        let register_pago_use_case = Arc::new(RegisterPagoUseCase::new(
            pago_repository.clone(),
            file_repository.clone()
        ));
        let update_pago_use_case = Arc::new(UpdatePagoUseCase::new(
            pago_repository.clone()
        ));
        
        Ok(Self {
            // Auth use cases
            login_use_case,
            logout_use_case,
            verify_session_use_case,
            // Persona use cases
            create_persona_use_case,
            update_persona_use_case,
            search_personas_use_case,
            // Agencia use cases
            create_agencia_use_case,
            update_agencia_use_case,
            deactivate_agencia_use_case,
            restore_agencia_use_case,
            // Tour use cases
            create_tour_use_case,
            update_tour_use_case,
            search_tours_use_case,
            deactivate_tour_use_case,
            restore_tour_use_case,
            // File use cases
            create_file_use_case,
            update_file_use_case,
            search_files_use_case,
            // Pago use cases
            register_pago_use_case,
            update_pago_use_case,
            // Services
            session_manager,
            password_hasher,
            logging_service,
            notification_service,
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
                        info!("✅ Tigris Storage inicializado correctamente");
                    }
                    Err(e) => {
                        warn!("⚠️ No se pudo inicializar Tigris Storage: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("⚠️ Tigris Storage no configurado: {}", e);
            }
        }
    }
}