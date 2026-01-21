-- ========================================================================
-- ROLLBACK: Revertir cambios de file_guias, file_vehiculos, file_entradas, entradas
-- ========================================================================

-- 4) Revertir entradas: quitar tours_asociados, restaurar ruta
DROP INDEX IF EXISTS idx_entradas_tours_asociados;
ALTER TABLE entradas DROP COLUMN IF EXISTS tours_asociados;
ALTER TABLE entradas ADD COLUMN ruta VARCHAR(200);

-- 3) Quitar id_entrada_precio de file_entradas
DROP INDEX IF EXISTS idx_file_entradas_precio;
ALTER TABLE file_entradas DROP COLUMN IF EXISTS id_entrada_precio;

-- 2) Revertir file_vehiculos: restaurar id_file
DROP INDEX IF EXISTS idx_file_vehiculos_id_file_tour;
ALTER TABLE file_vehiculos DROP CONSTRAINT IF EXISTS uq_file_tour_vehiculos;
ALTER TABLE file_vehiculos ADD COLUMN id_file INTEGER;

-- Migrar datos de vuelta (si es posible)
UPDATE file_vehiculos fv
SET id_file = ft.id_file
FROM file_tours ft
WHERE ft.id = fv.id_file_tour;

ALTER TABLE file_vehiculos DROP COLUMN id_file_tour;
ALTER TABLE file_vehiculos ALTER COLUMN id_file SET NOT NULL;
ALTER TABLE file_vehiculos ADD CONSTRAINT fk_file_vehiculos_file FOREIGN KEY (id_file) REFERENCES files(id) ON DELETE CASCADE;
ALTER TABLE file_vehiculos ADD CONSTRAINT uq_file_vehiculos UNIQUE (id_file, id_vehiculo);
CREATE INDEX idx_file_vehiculos_id_file ON file_vehiculos(id_file);

-- 1) Revertir file_guias: restaurar id_file
DROP INDEX IF EXISTS idx_file_guias_id_file_tour;
ALTER TABLE file_guias DROP CONSTRAINT IF EXISTS uq_file_tour_guias;
ALTER TABLE file_guias ADD COLUMN id_file INTEGER;

-- Migrar datos de vuelta (si es posible)
UPDATE file_guias fg
SET id_file = ft.id_file
FROM file_tours ft
WHERE ft.id = fg.id_file_tour;

ALTER TABLE file_guias DROP COLUMN id_file_tour;
ALTER TABLE file_guias ALTER COLUMN id_file SET NOT NULL;
ALTER TABLE file_guias ADD CONSTRAINT fk_file_guias_file FOREIGN KEY (id_file) REFERENCES files(id) ON DELETE CASCADE;
ALTER TABLE file_guias ADD CONSTRAINT uq_file_guias UNIQUE (id_file, id_guia);
CREATE INDEX idx_file_guias_id_file ON file_guias(id_file);
