-- Restaurar media en vehiculos
ALTER TABLE vehiculos ADD COLUMN IF NOT EXISTS media JSONB;

-- Mover media de transportes de vuelta a vehiculos
UPDATE vehiculos v
SET media = t.media
FROM transportes t
WHERE v.id_transporte = t.id
  AND t.media IS NOT NULL;

-- Remover media de transportes
ALTER TABLE transportes DROP COLUMN IF EXISTS media;

-- Remover is_active de las tablas
ALTER TABLE conductores DROP COLUMN IF EXISTS is_active;
ALTER TABLE guias DROP COLUMN IF EXISTS is_active;
ALTER TABLE vehiculos DROP COLUMN IF EXISTS is_active;
ALTER TABLE files DROP COLUMN IF EXISTS is_active;
