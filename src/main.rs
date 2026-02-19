use std::net::SocketAddr;
use std::sync::Arc;

mod config;
mod domain;
mod application;
mod infrastructure;
mod presentation;

use config::AppConfig;
use infrastructure::persistence::database::DatabasePool;
use infrastructure::container::DependencyContainer;
use presentation::routes::create_router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Cargar variables de entorno
    dotenvy::dotenv().ok();
    
    // Inicializar tracing/logging con formato limpio y detallado
    // Similar al proyecto inventariado - formato simple sin filtros complejos
    tracing_subscriber::fmt()
        .with_target(true)      // Muestra el módulo/target
        .with_file(true)        // Muestra nombre de archivo
        .with_line_number(true) // Muestra número de línea
        .with_level(true)       // Muestra nivel (INFO, DEBUG, etc.)
        .with_ansi(true)        // Colores habilitados
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("secure_auth_api=debug".parse().unwrap())
                .add_directive("tower_http=warn".parse().unwrap())
                .add_directive("tokio_postgres=warn".parse().unwrap())
                .add_directive("deadpool=warn".parse().unwrap())
                .add_directive("diesel=warn".parse().unwrap())
                .add_directive("axum=warn".parse().unwrap())
                .add_directive("hyper=warn".parse().unwrap())
        )
        .init();
    
    tracing::info!("Starting Tour Operator Backend - Sistema de Gestión de Pasajeros");
    tracing::info!("Authentication: Session-based cookies");
    
    // Cargar y validar configuración
    let config = AppConfig::from_env()?;
    config.validate_security()?;
    tracing::info!("Configuration loaded and validated");
    
    // Inicializar pool de base de datos async (deadpool)
    let db_pool = DatabasePool::new(&config).await?;
    tracing::info!("Async database pool initialized (deadpool)");
    
    // Ejecutar migraciones
    db_pool.run_migrations().await?;
    tracing::info!("Database migrations completed");
    
    // Crear broadcaster de notificaciones (necesario antes del container)
    let broadcaster = Arc::new(infrastructure::sse::NotificationBroadcaster::new());
    tracing::info!("Notification broadcaster initialized");
    
    // Clonar db_pool para la tarea diaria (antes de moverlo al container)
    let db_pool_for_automation = db_pool.clone();

    // Crear contenedor de dependencias
    let mut container = DependencyContainer::new(db_pool, config.clone(), broadcaster.clone())?;
    
    // Inicializar storage de Tigris (async, opcional)
    container.init_storage().await;
    
    let container = Arc::new(container);
    tracing::info!("Dependency container initialized");
    
    // Iniciar tarea diaria de automatización de estados a las 00:00
    {
        let db_pool_clone = db_pool_for_automation;
        tokio::spawn(async move {
            loop {
                let now = chrono::Local::now();
                let tomorrow_midnight = (now.date_naive() + chrono::Duration::days(1))
                    .and_hms_opt(0, 0, 0)
                    .unwrap();
                let duration_until_midnight = tomorrow_midnight
                    .signed_duration_since(now.naive_local())
                    .to_std()
                    .unwrap_or(std::time::Duration::from_secs(60));

                tracing::info!(
                    "Daily status automation scheduled for next midnight (in {} seconds)",
                    duration_until_midnight.as_secs()
                );

                tokio::time::sleep(duration_until_midnight).await;

                tracing::info!("Running daily status automation...");
                match db_pool_clone.run_daily_automation().await {
                    Ok(()) => tracing::info!("Daily status automation completed successfully"),
                    Err(e) => tracing::error!("Daily status automation failed: {}", e),
                }
            }
        });
    }

    // Crear router con todas las rutas
    let app = create_router(container, broadcaster, &config);

    // Configurar dirección del servidor
    // Usar 0.0.0.0 para aceptar conexiones desde cualquier interfaz (necesario en Docker/Fly.io)
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Server listening on http://{}", addr);
    
    // Iniciar servidor
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
