//! Contenedor de Dependencias (DI Container)
//!
//! Orquesta la creación de todos los componentes del sistema:
//! repositorios, servicios, casos de uso y storage.

mod auth;
mod repositories;
mod services;

use std::sync::Arc;

use crate::application::ports::{
    UserRepositoryPort, PersonaRepositoryPort, AgenciaRepositoryPort,
    TourRepositoryPort, TransporteRepositoryPort, VehiculoRepositoryPort,
    ConductorRepositoryPort, GuiaRepositoryPort, RestauranteRepositoryPort,
    EntradaRepositoryPort, FileRepositoryPort,
    ActivityLogRepositoryPort, NotificationRepositoryPort,
    FileEntradaRepositoryPort, FileGuiaRepositoryPort, FilePasajeroRepositoryPort,
    FileRestauranteRepositoryPort, FileVehiculoRepositoryPort, FileTourRepositoryPort,
    SessionManagerPort, PasswordHasherPort, CachePort,
};
use crate::application::services::{
    LoggingService, NotificationService, UserService, AgenciaService,
    PersonaService, TourService, FileService, RestauranteService,
    TransporteService, VehiculoService, ConductorService, EntradaService,
    EntradaPrecioService, GuiaService, MyFilesService, ContabilidadService,
    FileAssignmentService, MisPagosService, SaldoFavorService,
};
use crate::application::use_cases::auth::{LoginUseCase, LogoutUseCase, VerifySessionUseCase};
use crate::config::AppConfig;
use crate::domain::errors::ApplicationError;
use crate::infrastructure::cache::AppCache;
use crate::infrastructure::persistence::DatabasePool;
use crate::infrastructure::sse::NotificationBroadcaster;

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
    pub restaurante_service: Arc<RestauranteService>,
    pub transporte_service: Arc<TransporteService>,
    pub vehiculo_service: Arc<VehiculoService>,
    pub conductor_service: Arc<ConductorService>,
    pub entrada_service: Arc<EntradaService>,
    pub entrada_precio_service: Arc<EntradaPrecioService>,
    pub guia_service: Arc<GuiaService>,
    pub my_files_service: Arc<MyFilesService>,
    pub contabilidad_service: Arc<ContabilidadService>,
    pub file_assignment_service: Arc<FileAssignmentService>,
    pub mis_pagos_service: Arc<MisPagosService>,
    pub saldo_favor_service: Arc<SaldoFavorService>,

    // Object Storage (Tigris) - Opcional
    pub tigris_storage: Option<Arc<crate::infrastructure::storage::TigrisStorage>>,

    // Entity Repositories
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
    pub activity_log_repository: Arc<dyn ActivityLogRepositoryPort>,
    pub notification_repository: Arc<dyn NotificationRepositoryPort>,

    // File Relations Repositories
    pub file_entrada_repository: Arc<dyn FileEntradaRepositoryPort>,
    pub file_guia_repository: Arc<dyn FileGuiaRepositoryPort>,
    pub file_pasajero_repository: Arc<dyn FilePasajeroRepositoryPort>,
    pub file_restaurante_repository: Arc<dyn FileRestauranteRepositoryPort>,
    pub file_vehiculo_repository: Arc<dyn FileVehiculoRepositoryPort>,
    pub file_tour_repository: Arc<dyn FileTourRepositoryPort>,

    // Cache
    pub cache: Arc<dyn CachePort>,

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
    pub fn new(
        db_pool: DatabasePool,
        config: AppConfig,
        broadcaster: Arc<NotificationBroadcaster>,
    ) -> Result<Self, ApplicationError> {
        // Validar configuración de seguridad
        config.validate_security()
            .map_err(|e| ApplicationError::Configuration(e.to_string()))?;

        // Crear caché centralizado
        let cache: Arc<dyn CachePort> = Arc::new(AppCache::new());

        // 1. Crear todos los repositorios
        let repos = repositories::Repositories::create(db_pool.clone(), cache.clone());

        // 2. Crear componentes de autenticación y seguridad
        let auth = auth::AuthComponents::create(
            &config,
            repos.user.clone(),
            repos.session.clone(),
        )?;

        // 3. Crear todos los servicios
        let svcs = services::Services::create(
            db_pool,
            &repos,
            auth.password_hasher.clone(),
            broadcaster,
        );

        Ok(Self {
            // Auth
            login_use_case: auth.login_use_case,
            logout_use_case: auth.logout_use_case,
            verify_session_use_case: auth.verify_session_use_case,
            session_manager: auth.session_manager,
            password_hasher: auth.password_hasher,
            // Services
            logging_service: svcs.logging,
            notification_service: svcs.notification,
            user_service: svcs.user,
            agencia_service: svcs.agencia,
            persona_service: svcs.persona,
            tour_service: svcs.tour,
            file_service: svcs.file,
            restaurante_service: svcs.restaurante,
            transporte_service: svcs.transporte,
            vehiculo_service: svcs.vehiculo,
            conductor_service: svcs.conductor,
            entrada_service: svcs.entrada,
            entrada_precio_service: svcs.entrada_precio,
            guia_service: svcs.guia,
            my_files_service: svcs.my_files,
            contabilidad_service: svcs.contabilidad,
            file_assignment_service: svcs.file_assignment,
            mis_pagos_service: svcs.mis_pagos,
            saldo_favor_service: svcs.saldo_favor,
            // Repositories
            user_repository: repos.user,
            persona_repository: repos.persona,
            agencia_repository: repos.agencia,
            tour_repository: repos.tour,
            transporte_repository: repos.transporte,
            vehiculo_repository: repos.vehiculo,
            conductor_repository: repos.conductor,
            guia_repository: repos.guia,
            restaurante_repository: repos.restaurante,
            entrada_repository: repos.entrada,
            file_repository: repos.file,
            activity_log_repository: repos.activity_log,
            notification_repository: repos.notification,
            // File Relations
            file_entrada_repository: repos.file_entrada,
            file_guia_repository: repos.file_guia,
            file_pasajero_repository: repos.file_pasajero,
            file_restaurante_repository: repos.file_restaurante,
            file_vehiculo_repository: repos.file_vehiculo,
            file_tour_repository: repos.file_tour,
            // Cache
            cache,
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
