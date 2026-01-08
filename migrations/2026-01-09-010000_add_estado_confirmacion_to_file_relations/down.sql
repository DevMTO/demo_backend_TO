-- ========================================================================
-- REVERTIR estado_confirmacion de file_guias y file_vehiculos
-- ========================================================================

-- Quitar constraints
ALTER TABLE file_guias DROP CONSTRAINT IF EXISTS chk_file_guias_estado;
ALTER TABLE file_vehiculos DROP CONSTRAINT IF EXISTS chk_file_vehiculos_estado;

-- Quitar índices
DROP INDEX IF EXISTS idx_file_guias_estado;
DROP INDEX IF EXISTS idx_file_vehiculos_estado;

-- Quitar columnas de file_guias
ALTER TABLE file_guias 
    DROP COLUMN IF EXISTS estado_confirmacion,
    DROP COLUMN IF EXISTS confirmado_at,
    DROP COLUMN IF EXISTS motivo_rechazo;

-- Quitar columnas de file_vehiculos
ALTER TABLE file_vehiculos 
    DROP COLUMN IF EXISTS estado_confirmacion,
    DROP COLUMN IF EXISTS confirmado_at,
    DROP COLUMN IF EXISTS motivo_rechazo;
