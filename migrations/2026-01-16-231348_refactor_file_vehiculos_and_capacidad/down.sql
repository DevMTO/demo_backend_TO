-- ========================================================================
-- ROLLBACK: Revertir cambios de refactorización
-- ========================================================================

-- PASO 1: Restaurar capacidad_disponible en vehiculos
ALTER TABLE vehiculos
ADD COLUMN capacidad_disponible INTEGER NOT NULL DEFAULT 0;

-- Restaurar constraints de vehiculos
ALTER TABLE vehiculos
ADD CONSTRAINT check_capacidad_disponible_positive CHECK (capacidad_disponible >= 0);

ALTER TABLE vehiculos
ADD CONSTRAINT check_capacidad_disponible_max CHECK (capacidad_disponible <= capacidad);

-- Actualizar con el valor de capacidad
UPDATE vehiculos SET capacidad_disponible = capacidad;

-- PASO 2: Quitar capacidad_asignada de file_vehiculos
ALTER TABLE file_vehiculos DROP CONSTRAINT IF EXISTS check_capacidad_asignada_positive;
ALTER TABLE file_vehiculos DROP COLUMN IF EXISTS capacidad_asignada;

-- PASO 3: Restaurar campos de confirmación en file_vehiculos
ALTER TABLE file_vehiculos 
    ADD COLUMN estado_confirmacion VARCHAR(20) NOT NULL DEFAULT 'pendiente',
    ADD COLUMN confirmado_at TIMESTAMPTZ NULL,
    ADD COLUMN motivo_rechazo TEXT NULL;

-- Restaurar índice
CREATE INDEX idx_file_vehiculos_estado ON file_vehiculos(estado_confirmacion);

-- Restaurar constraint
ALTER TABLE file_vehiculos 
    ADD CONSTRAINT chk_file_vehiculos_estado 
    CHECK (estado_confirmacion IN ('pendiente', 'aceptado', 'rechazado'));
