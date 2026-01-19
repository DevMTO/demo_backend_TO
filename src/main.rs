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
                .add_directive("secure_auth_api=info".parse().unwrap())
                .add_directive("tower_http=warn".parse().unwrap())
                .add_directive("tokio_postgres=error".parse().unwrap())
                .add_directive("deadpool=error".parse().unwrap())
                .add_directive("axum=warn".parse().unwrap())
                .add_directive("hyper=warn".parse().unwrap())
        )
        .init();
    
    tracing::info!("🚀 Starting Tour Operator Backend - Sistema de Gestión de Pasajeros");
    tracing::info!("🔒 Authentication: Session-based cookies");
    
    // Cargar y validar configuración
    let config = AppConfig::from_env()?;
    config.validate_security()?;
    tracing::info!("✅ Configuration loaded and validated");
    
    // Inicializar pool de base de datos async (deadpool)
    let db_pool = DatabasePool::new(&config).await?;
    tracing::info!("✅ Async database pool initialized (deadpool)");
    
    // Ejecutar migraciones
    db_pool.run_migrations().await?;
    tracing::info!("✅ Database migrations completed");
    
    // Crear broadcaster de notificaciones (necesario antes del container)
    let broadcaster = Arc::new(infrastructure::sse::NotificationBroadcaster::new());
    tracing::info!("✅ Notification broadcaster initialized");
    
    // Crear contenedor de dependencias
    let mut container = DependencyContainer::new(db_pool, config.clone(), broadcaster.clone())?;
    
    // Inicializar storage de Tigris (async, opcional)
    container.init_storage().await;
    
    let container = Arc::new(container);
    tracing::info!("✅ Dependency container initialized");
    
    // Crear router con todas las rutas
    let app = create_router(container, broadcaster, &config);
    
    // Configurar dirección del servidor
    // Usar 0.0.0.0 para aceptar conexiones desde cualquier interfaz (necesario en Docker/Fly.io)
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🌐 Server listening on http://{}", addr);
    
    // Iniciar servidor
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
