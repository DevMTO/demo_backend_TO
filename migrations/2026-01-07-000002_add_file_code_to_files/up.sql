-- ========================================================================
-- AGREGAR file_code A FILES
-- Código único para identificación de archivos/expedientes (opcional)
-- ========================================================================

-- Agregar columna file_code (nullable, sin valor por defecto)
ALTER TABLE files ADD COLUMN IF NOT EXISTS file_code VARCHAR(50) UNIQUE;

-- Crear índice para búsquedas rápidas por file_code
CREATE INDEX IF NOT EXISTS idx_files_file_code ON files(file_code);
