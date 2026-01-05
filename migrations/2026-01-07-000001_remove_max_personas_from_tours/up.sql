-- ========================================================================
-- ELIMINAR COLUMNA max_personas DE TOURS
-- Esta columna no es necesaria en el modelo actual
-- ========================================================================

ALTER TABLE tours DROP COLUMN IF EXISTS max_personas;
