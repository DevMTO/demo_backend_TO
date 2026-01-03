-- Agregar media a transportes (mover de vehiculos)
ALTER TABLE transportes ADD COLUMN IF NOT EXISTS media JSONB;

-- Agregar is_active a conductores
ALTER TABLE conductores ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true;

-- Agregar is_active a guias
ALTER TABLE guias ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true;

-- Agregar is_active a vehiculos (ya tiene status pero no is_active)
ALTER TABLE vehiculos ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true;

-- Agregar is_active a files (ya tiene status pero no is_active)
ALTER TABLE files ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true;

-- Actualizar is_active basado en status existente para conductores
UPDATE conductores SET is_active = (status = 'activo');

-- Actualizar is_active basado en status existente para guias
UPDATE guias SET is_active = (status = 'activo');

-- Actualizar is_active basado en status existente para vehiculos
UPDATE vehiculos SET is_active = (status = 'activo');

-- Mover datos de media de vehiculos a transportes (un vehículo por transporte)
-- Solo si el transporte no tiene media aún
UPDATE transportes t
SET media = v.media
FROM vehiculos v
WHERE v.id_transporte = t.id
  AND v.media IS NOT NULL
  AND t.media IS NULL;

-- Limpiar media de vehiculos (ahora está en transportes)
ALTER TABLE vehiculos DROP COLUMN IF EXISTS media;
