-- ========================================================================
-- ELIMINAR file_code DE FILES
-- ========================================================================

-- Eliminar índice
DROP INDEX IF EXISTS idx_files_file_code;

-- Eliminar columna
ALTER TABLE files DROP COLUMN IF EXISTS file_code;
