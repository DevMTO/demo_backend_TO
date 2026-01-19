-- ========================================================================
-- MIGRACIÓN: Refactorizar file_vehiculos y mover capacidad_disponible
-- 
-- Cambios:
-- 1. Quitar estado_confirmacion, confirmado_at, motivo_rechazo de file_vehiculos
--    (solo se usa para guías)
-- 2. Mover capacidad_disponible de vehiculos a file_vehiculos
--    (tiene más sentido por asignación específica)
-- ========================================================================

-- ========================================================================
-- PASO 1: Quitar campos de confirmación de file_vehiculos
-- ========================================================================

-- Primero eliminar los constraints
ALTER TABLE file_vehiculos DROP CONSTRAINT IF EXISTS chk_file_vehiculos_estado;

-- Eliminar el índice
DROP INDEX IF EXISTS idx_file_vehiculos_estado;

-- Eliminar las columnas
ALTER TABLE file_vehiculos DROP COLUMN IF EXISTS estado_confirmacion;
ALTER TABLE file_vehiculos DROP COLUMN IF EXISTS confirmado_at;
ALTER TABLE file_vehiculos DROP COLUMN IF EXISTS motivo_rechazo;

-- ========================================================================
-- PASO 2: Agregar capacidad_asignada a file_vehiculos
-- (Representa cuántos asientos se asignan en esta relación file-vehiculo)
-- ========================================================================

-- Agregar columna capacidad_asignada (por defecto 0, se setea al crear)
ALTER TABLE file_vehiculos
ADD COLUMN capacidad_asignada INTEGER NOT NULL DEFAULT 0;

-- Agregar constraint para que no sea negativa
ALTER TABLE file_vehiculos
ADD CONSTRAINT check_capacidad_asignada_positive CHECK (capacidad_asignada >= 0);

-- Comentario descriptivo
COMMENT ON COLUMN file_vehiculos.capacidad_asignada IS 'Cantidad de asientos asignados a este file desde el vehículo';

-- ========================================================================
-- PASO 3: Quitar capacidad_disponible de vehiculos
-- (Ya no tiene sentido estar en vehiculos, se calcula dinámicamente)
-- ========================================================================

-- Eliminar constraints
ALTER TABLE vehiculos DROP CONSTRAINT IF EXISTS check_capacidad_disponible_positive;
ALTER TABLE vehiculos DROP CONSTRAINT IF EXISTS check_capacidad_disponible_max;

-- Eliminar la columna
ALTER TABLE vehiculos DROP COLUMN IF EXISTS capacidad_disponible;
