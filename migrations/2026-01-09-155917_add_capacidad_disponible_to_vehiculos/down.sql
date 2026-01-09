-- Revertir migración: Eliminar capacidad_disponible de vehiculos

-- Eliminar constraints
ALTER TABLE vehiculos DROP CONSTRAINT IF EXISTS check_capacidad_disponible_max;
ALTER TABLE vehiculos DROP CONSTRAINT IF EXISTS check_capacidad_disponible_positive;

-- Eliminar columna
ALTER TABLE vehiculos DROP COLUMN IF EXISTS capacidad_disponible;
