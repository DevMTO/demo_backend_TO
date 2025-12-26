//! # Async Database Connection Pool
//! 
//! Pool de conexiones asíncronas a PostgreSQL usando diesel-async y deadpool.


use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use diesel::pg::PgConnection;
use diesel::Connection;

use crate::config::AppConfig;
use crate::domain::errors::ApplicationError;

/// Migraciones embebidas
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Tipo del pool asíncrono
pub type DbPool = Pool<AsyncPgConnection>;

/// Pool de conexiones asíncronas a la base de datos
#[derive(Clone)]
pub struct DatabasePool {
    pool: DbPool,
    database_url: String,
}

impl DatabasePool {
    /// Crear un nuevo pool de conexiones asíncronas
    pub async fn new(config: &AppConfig) -> Result<Self, ApplicationError> {
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database_url);
        
        let pool = Pool::builder(manager)
            .max_size(config.database_max_connections as usize)
            .build()
            .map_err(|e| ApplicationError::Configuration(format!("Database pool error: {}", e)))?;
        
        // Verificar conexión
        let _conn = pool.get().await
            .map_err(|e| ApplicationError::Configuration(format!("Database connection test failed: {}", e)))?;
        
        Ok(Self { 
            pool,
            database_url: config.database_url.clone(),
        })
    }
    
    /// Obtener una conexión del pool
    pub async fn get_connection(&self) -> Result<deadpool::managed::Object<AsyncDieselConnectionManager<AsyncPgConnection>>, ApplicationError> {
        self.pool
            .get()
            .await
            .map_err(|e| ApplicationError::Repository(format!("Connection error: {}", e)))
    }
    
    /// Obtener el pool interno
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
    
    /// Ejecutar migraciones (usa conexión síncrona temporal)
    pub async fn run_migrations(&self) -> Result<(), ApplicationError> {
        // Las migraciones de Diesel necesitan una conexión síncrona
        let database_url = self.database_url.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut conn = PgConnection::establish(&database_url)
                .map_err(|e| ApplicationError::Configuration(format!("Migration connection error: {}", e)))?;
            
            conn.run_pending_migrations(MIGRATIONS)
                .map_err(|e| ApplicationError::Configuration(format!("Migration error: {}", e)))?;
            
            Ok::<(), ApplicationError>(())
        })
        .await
        .map_err(|e| ApplicationError::Configuration(format!("Migration task error: {}", e)))??;
        
        Ok(())
    }
}
