//! Creación de servicios de negocio para el contenedor de dependencias

use std::sync::Arc;

use crate::application::ports::NotificationServicePort;
use crate::application::services::{
    LoggingService, NotificationService, UserService, AgenciaService,
    PersonaService, TourService, FileService, PagoService, RestauranteService,
    TransporteService, VehiculoService, ConductorService, EntradaService,
    EntradaPrecioService, GuiaService, MyFilesService, PostgresMyFilesRepository,
    ContabilidadService, FileAssignmentService, MisPagosService, PostgresMisPagosRepository,
};
use crate::infrastructure::persistence::DatabasePool;
use crate::infrastructure::NotificationBroadcastAdapter;
use crate::infrastructure::sse::NotificationBroadcaster;

use super::repositories::Repositories;

/// Conjunto de todos los servicios del sistema
pub(super) struct Services {
    // System
    pub logging: Arc<LoggingService>,
    pub notification: Arc<NotificationService>,
    // Business
    pub user: Arc<UserService>,
    pub agencia: Arc<AgenciaService>,
    pub persona: Arc<PersonaService>,
    pub tour: Arc<TourService>,
    pub file: Arc<FileService>,
    pub pago: Arc<PagoService>,
    pub restaurante: Arc<RestauranteService>,
    pub transporte: Arc<TransporteService>,
    pub vehiculo: Arc<VehiculoService>,
    pub conductor: Arc<ConductorService>,
    pub entrada: Arc<EntradaService>,
    pub entrada_precio: Arc<EntradaPrecioService>,
    pub guia: Arc<GuiaService>,
    pub my_files: Arc<MyFilesService>,
    pub contabilidad: Arc<ContabilidadService>,
    pub file_assignment: Arc<FileAssignmentService>,
    pub mis_pagos: Arc<MisPagosService>,
}

impl Services {
    pub fn create(
        db_pool: DatabasePool,
        repos: &Repositories,
        password_hasher: Arc<dyn crate::application::ports::PasswordHasherPort>,
        broadcaster: Arc<NotificationBroadcaster>,
    ) -> Self {
        // Servicios de sistema
        let logging = Arc::new(LoggingService::new(repos.activity_log.clone()));
        let notification = Arc::new(NotificationService::new(repos.notification.clone()));

        // Adaptador de notificaciones con broadcast SSE
        let notify: Arc<dyn NotificationServicePort> = Arc::new(
            NotificationBroadcastAdapter::new(
                notification.clone(),
                repos.notification.clone(),
                broadcaster,
            )
        );

        // Servicios de negocio
        let user = Arc::new(UserService::new(
            repos.user.clone(),
            repos.persona.clone(),
            password_hasher,
            logging.clone(),
            notify.clone(),
        ));

        let agencia = Arc::new(AgenciaService::new(
            repos.agencia.clone(),
            logging.clone(),
            notify.clone(),
        ));

        let persona = Arc::new(PersonaService::new(
            repos.persona.clone(),
            logging.clone(),
        ));

        let tour = Arc::new(TourService::new(
            repos.tour.clone(),
            logging.clone(),
            notify.clone(),
        ));

        let file = Arc::new(FileService::new(
            repos.file.clone(),
            repos.file_tour.clone(),
            logging.clone(),
            notify.clone(),
            repos.pago_file.clone(),
            repos.agencia.clone(),
        ));

        let pago = Arc::new(PagoService::new(
            repos.pago.clone(),
            repos.file.clone(),
            logging.clone(),
            notify.clone(),
        ));

        let restaurante = Arc::new(RestauranteService::new(
            repos.restaurante.clone(),
            logging.clone(),
            notify.clone(),
        ));

        let transporte = Arc::new(TransporteService::new(
            repos.transporte.clone(),
            logging.clone(),
            notify.clone(),
        ));

        let vehiculo = Arc::new(VehiculoService::new(
            repos.vehiculo.clone(),
            logging.clone(),
            notify.clone(),
        ));

        let conductor = Arc::new(ConductorService::new(
            repos.conductor.clone(),
            logging.clone(),
            notify.clone(),
        ));

        let entrada = Arc::new(EntradaService::new(
            repos.entrada.clone(),
            repos.entrada_precio.clone(),
            logging.clone(),
            notify.clone(),
        ));

        let entrada_precio = Arc::new(EntradaPrecioService::new(
            repos.entrada_precio.clone(),
        ));

        let guia = Arc::new(GuiaService::new(
            repos.guia.clone(),
            logging.clone(),
            notify.clone(),
        ));

        let my_files_repo = Arc::new(PostgresMyFilesRepository::new(db_pool.clone()));
        let my_files = Arc::new(MyFilesService::new(my_files_repo));

        let contabilidad = Arc::new(ContabilidadService::new(
            repos.cuenta.clone(),
            repos.movimiento.clone(),
            repos.pago_file.clone(),
            repos.pago_proveedor.clone(),
            repos.tarifa_servicio.clone(),
            repos.agencia.clone(),
            repos.file.clone(),
            notify.clone(),
        ));

        let file_assignment = Arc::new(FileAssignmentService::new(
            repos.file.clone(),
            repos.file_tour.clone(),
            repos.file_vehiculo.clone(),
            repos.file_guia.clone(),
            repos.file_restaurante.clone(),
            repos.file_entrada.clone(),
            repos.conductor.clone(),
            repos.guia.clone(),
            repos.user.clone(),
            logging.clone(),
            notify,
        ));

        let mis_pagos_repo = Arc::new(PostgresMisPagosRepository::new(db_pool.clone()));
        let mis_pagos = Arc::new(MisPagosService::new(mis_pagos_repo));

        Self {
            logging, notification, user, agencia, persona, tour, file, pago,
            restaurante, transporte, vehiculo, conductor, entrada,
            entrada_precio, guia, my_files, contabilidad, file_assignment,
            mis_pagos,
        }
    }
}
