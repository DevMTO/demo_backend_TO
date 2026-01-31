-- ========================================================================
-- ROLLBACK: Remover geolocalización de tours y file_tours
-- ========================================================================

-- Remover constraints
ALTER TABLE file_tours DROP CONSTRAINT IF EXISTS chk_file_tours_geo_recojo_valid;
ALTER TABLE tours DROP CONSTRAINT IF EXISTS chk_tours_geo_fin_valid;
ALTER TABLE tours DROP CONSTRAINT IF EXISTS chk_tours_geo_inicio_valid;

-- Remover función de validación
DROP FUNCTION IF EXISTS is_valid_geo_point(JSONB);

-- Remover índices
DROP INDEX IF EXISTS idx_file_tours_geo_recojo;
DROP INDEX IF EXISTS idx_tours_geo_fin;
DROP INDEX IF EXISTS idx_tours_geo_inicio;

-- Remover columna de file_tours
ALTER TABLE file_tours DROP COLUMN IF EXISTS geo_recojo;

-- Restaurar NOT NULL en tours (antes de remover columnas geo)
-- Nota: Esto puede fallar si hay datos NULL, se deberá limpiar primero
UPDATE tours SET lugar_inicio = 'Sin especificar' WHERE lugar_inicio IS NULL;
UPDATE tours SET lugar_fin = 'Sin especificar' WHERE lugar_fin IS NULL;
ALTER TABLE tours ALTER COLUMN lugar_inicio SET NOT NULL;
ALTER TABLE tours ALTER COLUMN lugar_fin SET NOT NULL;

-- Remover columnas geo de tours
ALTER TABLE tours DROP COLUMN IF EXISTS geo_ruta;
ALTER TABLE tours DROP COLUMN IF EXISTS geo_fin;
ALTER TABLE tours DROP COLUMN IF EXISTS geo_inicio;
