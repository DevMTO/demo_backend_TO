use s3::{Bucket, Region};
use s3::creds::Credentials;
use std::sync::Arc;
use tracing::{info, error, debug};

/// Configuración del servicio Tigris
#[derive(Debug, Clone)]
pub struct TigrisConfig {
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub endpoint: String,
    pub prefix: String,
}

impl TigrisConfig {
    /// Crea una configuración desde variables de entorno
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            access_key: std::env::var("TIGRIS_ACCESS_KEY")
                .map_err(|_| "TIGRIS_ACCESS_KEY no configurado")?,
            secret_key: std::env::var("TIGRIS_SECRET_KEY")
                .map_err(|_| "TIGRIS_SECRET_KEY no configurado")?,
            bucket_name: std::env::var("TIGRIS_BUCKET")
                .map_err(|_| "TIGRIS_BUCKET no configurado")?,
            endpoint: std::env::var("TIGRIS_ENDPOINT")
                .map_err(|_| "TIGRIS_ENDPOINT no configurado")?,
            prefix: std::env::var("TIGRIS_PREFIX")
                .unwrap_or_else(|_| "uploads".to_string()),
        })
    }
}

/// Servicio de Storage para operaciones con Tigris
#[derive(Clone)]
pub struct TigrisStorage {
    bucket: Arc<Bucket>,
    prefix: String,
}

impl TigrisStorage {
    /// Crea una nueva instancia del servicio de storage
    pub async fn new(config: TigrisConfig) -> Result<Self, String> {
        // Crear credenciales
        let credentials = Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None, // security_token
            None, // session_token
            None, // profile
        ).map_err(|e| format!("Error creando credenciales: {}", e))?;

        // Configurar región custom (Tigris endpoint)
        let region = Region::Custom {
            region: "auto".to_string(),
            endpoint: config.endpoint.clone(),
        };

        // Crear bucket
        let bucket = Bucket::new(&config.bucket_name, region, credentials)
            .map_err(|e| format!("Error creando bucket: {}", e))?
            .with_path_style();

        info!("Tigris Storage inicializado: bucket={}, endpoint={}", 
            config.bucket_name, config.endpoint);

        Ok(Self {
            bucket: Arc::new(*bucket),
            prefix: config.prefix,
        })
    }

    /// Genera la ruta completa del archivo en el bucket
    fn get_full_path(&self, path: &str) -> String {
        if self.prefix.is_empty() {
            path.to_string()
        } else if path.starts_with(&self.prefix) {
            // El path ya contiene el prefix, no duplicar
            path.to_string()
        } else if path.starts_with(&format!("{}/", self.prefix)) {
            // El path ya contiene el prefix con slash, no duplicar
            path.to_string()
        } else {
            format!("{}/{}", self.prefix, path)
        }
    }

    /// Sube un archivo al storage
    /// 
    /// # Arguments
    /// * `path` - Ruta relativa dentro del bucket (ej: "agencias/1/logo.png")
    /// * `data` - Bytes del archivo
    /// * `content_type` - Tipo MIME del archivo
    /// 
    /// # Returns
    /// URL pública del archivo subido
    pub async fn upload(
        &self,
        path: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<String, String> {
        let full_path = self.get_full_path(path);
        
        debug!("📤 Subiendo archivo: {} ({} bytes)", full_path, data.len());

        let response = self.bucket
            .put_object_with_content_type(&full_path, data, content_type)
            .await
            .map_err(|e| format!("Error subiendo archivo: {}", e))?;

        if response.status_code() != 200 {
            error!("Error subiendo archivo: status={}", response.status_code());
            return Err(format!("Error HTTP: {}", response.status_code()));
        }

        // Generar URL del archivo
        // Tigris expone archivos públicos en: https://fly.storage.tigris.dev/{bucket}/{path}
        let url = format!(
            "{}/{}/{}",
            self.bucket.host(),
            self.bucket.name(),
            full_path
        );

        info!("Archivo subido: {}", url);
        Ok(url)
    }

    /// Obtiene un archivo del storage
    /// 
    /// # Arguments
    /// * `path` - Ruta relativa del archivo
    /// 
    /// # Returns
    /// Tupla (bytes, content_type)
    pub async fn get(&self, path: &str) -> Result<(Vec<u8>, String), String> {
        let full_path = self.get_full_path(path);
        
        debug!("📥 Descargando archivo: {}", full_path);

        let response = self.bucket
            .get_object(&full_path)
            .await
            .map_err(|e| format!("Error obteniendo archivo: {}", e))?;

        if response.status_code() != 200 {
            return Err(format!("Archivo no encontrado: {}", full_path));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        Ok((response.to_vec(), content_type))
    }

    /// Elimina un archivo del storage
    #[allow(dead_code)]
    pub async fn delete(&self, path: &str) -> Result<(), String> {
        let full_path = self.get_full_path(path);
        
        debug!("🗑️ Eliminando archivo: {}", full_path);

        let response = self.bucket
            .delete_object(&full_path)
            .await
            .map_err(|e| format!("Error eliminando archivo: {}", e))?;

        if response.status_code() != 204 && response.status_code() != 200 {
            return Err(format!("Error eliminando: status={}", response.status_code()));
        }

        info!("Archivo eliminado: {}", full_path);
        Ok(())
    }

    /// Genera una URL para proxy del archivo
    /// Esta URL pasa por nuestro backend para servir el archivo
    #[allow(dead_code)]
    pub fn get_proxy_url(&self, path: &str, base_url: &str) -> String {
        format!("{}/api/v1/storage/proxy/{}", base_url, path)
    }

    /// Verifica si un archivo existe
    #[allow(dead_code)]
    pub async fn exists(&self, path: &str) -> bool {
        let full_path = self.get_full_path(path);
        
        match self.bucket.head_object(&full_path).await {
            Ok((_, code)) => code == 200,
            Err(_) => false,
        }
    }

    /// Genera un path único para un archivo de agencia
    /// 
    /// # Arguments
    /// * `agencia_id` - ID de la agencia
    /// * `file_type` - Tipo de archivo (logo, banner, image)
    /// * `extension` - Extensión del archivo (png, jpg, etc)
    pub fn generate_agencia_path(agencia_id: i32, file_type: &str, extension: &str) -> String {
        let timestamp = chrono::Utc::now().timestamp();
        format!("agencias/{}/{}-{}.{}", agencia_id, file_type, timestamp, extension)
    }
    
    /// Genera un path único para un archivo de transporte
    /// 
    /// # Arguments
    /// * `transporte_id` - ID del transporte
    /// * `file_type` - Tipo de archivo (logo, banner, image)
    /// * `extension` - Extensión del archivo (png, jpg, etc)
    pub fn generate_transporte_path(transporte_id: i32, file_type: &str, extension: &str) -> String {
        let timestamp = chrono::Utc::now().timestamp();
        format!("transportes/{}/{}-{}.{}", transporte_id, file_type, timestamp, extension)
    }
    
    /// Genera un path único para un archivo de tour
    /// 
    /// # Arguments
    /// * `tour_id` - ID del tour
    /// * `file_type` - Tipo de archivo (image, cover, gallery)
    /// * `extension` - Extensión del archivo (png, jpg, etc)
    pub fn generate_tour_path(tour_id: i32, file_type: &str, extension: &str) -> String {
        let timestamp = chrono::Utc::now().timestamp();
        format!("tours/{}/{}-{}.{}", tour_id, file_type, timestamp, extension)
    }
}

/// Tipos de archivos permitidos para agencias
pub const ALLOWED_IMAGE_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg", 
    "image/jpg",
    "image/webp",
    "image/avif",
    "image/gif",
    "image/svg+xml",
];

/// Tamaño máximo de archivo (5 MB)
pub const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;

/// Valida el tipo de archivo
pub fn validate_content_type(content_type: &str) -> Result<(), String> {
    if ALLOWED_IMAGE_TYPES.contains(&content_type) {
        Ok(())
    } else {
        Err(format!(
            "Tipo de archivo no permitido: {}. Permitidos: {:?}",
            content_type, ALLOWED_IMAGE_TYPES
        ))
    }
}

/// Obtiene la extensión desde el content-type
pub fn extension_from_content_type(content_type: &str) -> &str {
    match content_type {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/webp" => "webp",
        "image/avif" => "avif",
        "image/gif" => "gif",
        "image/svg+xml" => "svg",
        _ => "bin",
    }
}
