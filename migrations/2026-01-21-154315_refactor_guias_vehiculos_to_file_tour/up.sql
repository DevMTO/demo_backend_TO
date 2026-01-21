-- ========================================================================
-- MIGRACIÓN: Refactorizar file_guias, file_vehiculos a file_tours
--            Agregar id_entrada_precio a file_entradas
--            Modificar entradas: quitar ruta, agregar tours_asociados
-- ========================================================================

-- ========================================================================
-- 1) MODIFICAR file_guias: cambiar id_file por id_file_tour
-- ========================================================================

-- Agregar nueva columna
ALTER TABLE file_guias ADD COLUMN id_file_tour INTEGER REFERENCES file_tours(id) ON DELETE CASCADE;

-- Migrar datos: buscar el primer file_tour por id_file
UPDATE file_guias fg
SET id_file_tour = ft.id
FROM file_tours ft
WHERE ft.id_file = fg.id_file
  AND ft.orden = 1;

-- Eliminar registros huérfanos (sin file_tour correspondiente)
DELETE FROM file_guias WHERE id_file_tour IS NULL;

-- Hacer la columna obligatoria
ALTER TABLE file_guias ALTER COLUMN id_file_tour SET NOT NULL;

-- Eliminar la columna id_file
ALTER TABLE file_guias DROP COLUMN id_file;

-- Eliminar constraint único antiguo y crear uno nuevo basado en file_tour
ALTER TABLE file_guias DROP CONSTRAINT IF EXISTS uq_file_guias;
ALTER TABLE file_guias ADD CONSTRAINT uq_file_tour_guias UNIQUE (id_file_tour, id_guia);

-- Actualizar índices
DROP INDEX IF EXISTS idx_file_guias_id_file;
CREATE INDEX idx_file_guias_id_file_tour ON file_guias(id_file_tour);

COMMENT ON COLUMN file_guias.id_file_tour IS 'FK al tour específico donde se asigna el guía';

-- ========================================================================
-- 2) MODIFICAR file_vehiculos: cambiar id_file por id_file_tour
-- ========================================================================

-- Agregar nueva columna
ALTER TABLE file_vehiculos ADD COLUMN id_file_tour INTEGER REFERENCES file_tours(id) ON DELETE CASCADE;

-- Migrar datos: buscar el primer file_tour por id_file
UPDATE file_vehiculos fv
SET id_file_tour = ft.id
FROM file_tours ft
WHERE ft.id_file = fv.id_file
  AND ft.orden = 1;

-- Eliminar registros huérfanos (sin file_tour correspondiente)
DELETE FROM file_vehiculos WHERE id_file_tour IS NULL;

-- Hacer la columna obligatoria
ALTER TABLE file_vehiculos ALTER COLUMN id_file_tour SET NOT NULL;

-- Eliminar la columna id_file
ALTER TABLE file_vehiculos DROP COLUMN id_file;

-- Eliminar constraint único antiguo y crear uno nuevo basado en file_tour
ALTER TABLE file_vehiculos DROP CONSTRAINT IF EXISTS uq_file_vehiculos;
ALTER TABLE file_vehiculos ADD CONSTRAINT uq_file_tour_vehiculos UNIQUE (id_file_tour, id_vehiculo);

-- Actualizar índices
DROP INDEX IF EXISTS idx_file_vehiculos_id_file;
CREATE INDEX idx_file_vehiculos_id_file_tour ON file_vehiculos(id_file_tour);

COMMENT ON COLUMN file_vehiculos.id_file_tour IS 'FK al tour específico donde se asigna el vehículo';

-- ========================================================================
-- 3) AGREGAR id_entrada_precio a file_entradas
-- ========================================================================

-- Agregar columna para precio específico de entrada
ALTER TABLE file_entradas ADD COLUMN id_entrada_precio INTEGER REFERENCES entrada_precios(id) ON DELETE SET NULL;

CREATE INDEX idx_file_entradas_precio ON file_entradas(id_entrada_precio) WHERE id_entrada_precio IS NOT NULL;

COMMENT ON COLUMN file_entradas.id_entrada_precio IS 'FK al precio específico elegido para esta entrada (opcional, permite calcular costos por rango de edad/tipo)';

-- ========================================================================
-- 4) MODIFICAR entradas: quitar ruta, agregar tours_asociados
-- ========================================================================

-- Eliminar columna ruta
ALTER TABLE entradas DROP COLUMN IF EXISTS ruta;
DROP INDEX IF EXISTS idx_entradas_ruta;

-- Agregar columna tours_asociados (JSONB con array de IDs de tours)
ALTER TABLE entradas ADD COLUMN tours_asociados JSONB DEFAULT NULL;

CREATE INDEX idx_entradas_tours_asociados ON entradas USING GIN (tours_asociados) WHERE tours_asociados IS NOT NULL;

COMMENT ON COLUMN entradas.tours_asociados IS 'Array JSON de IDs de tours que pueden usar esta entrada. Ej: [1, 3, 5]. NULL = disponible para todos los tours.';
