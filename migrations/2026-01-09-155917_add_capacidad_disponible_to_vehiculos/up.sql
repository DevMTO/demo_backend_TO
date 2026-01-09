-- ========================================================================
-- MIGRACIÓN: Agregar capacidad_disponible a vehiculos
-- Permite trackear la capacidad disponible actual del vehículo
-- ========================================================================

-- Agregar columna capacidad_disponible (inicialmente igual a capacidad)
ALTER TABLE vehiculos
ADD COLUMN capacidad_disponible INTEGER NOT NULL DEFAULT 0;

-- Actualizar registros existentes: capacidad_disponible = capacidad
UPDATE vehiculos SET capacidad_disponible = capacidad WHERE capacidad_disponible = 0;

-- Agregar constraint para que capacidad_disponible no sea negativa
ALTER TABLE vehiculos
ADD CONSTRAINT check_capacidad_disponible_positive CHECK (capacidad_disponible >= 0);

-- Agregar constraint para que capacidad_disponible no exceda capacidad
ALTER TABLE vehiculos
ADD CONSTRAINT check_capacidad_disponible_max CHECK (capacidad_disponible <= capacidad);

-- Comentario descriptivo
COMMENT ON COLUMN vehiculos.capacidad_disponible IS 'Capacidad disponible actual del vehículo (puede variar según asignaciones)';
