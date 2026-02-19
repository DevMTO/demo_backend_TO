-- ========================================================================
-- MIGRACIÓN: Drop file confirmado cascade trigger
--
-- Elimina:
-- - trg_files_status_change trigger en files table
-- - trg_after_file_status_change() function
--
-- Esto removes the cascade: file 'confirmado' → file_tours 'confirmado' → sub-files 'asignado'
-- ========================================================================

-- ========================================================================
-- 1) DROP TRIGGER en files
-- ========================================================================
DROP TRIGGER IF EXISTS trg_files_status_change ON files;

-- ========================================================================
-- 2) DROP TRIGGER FUNCTION
-- ========================================================================
DROP FUNCTION IF EXISTS trg_after_file_status_change();
