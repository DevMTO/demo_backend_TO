-- ========================================================================
-- ROLLBACK: Revertir modificaciones a file_pasajeros, mover campos de vuelta
--           y eliminar tiene_restaurante de tours
-- ========================================================================

-- =======================================================================
-- PARTE 1: Eliminar tiene_restaurante de tours
-- =======================================================================

ALTER TABLE tours DROP COLUMN IF EXISTS tiene_restaurante;

-- =======================================================================
-- PARTE 2: Restaurar campos en files (antes de eliminarlos de file_tours)
-- =======================================================================

ALTER TABLE files ADD COLUMN turno_tour VARCHAR(30);
ALTER TABLE files ADD COLUMN lugar_recojo VARCHAR(200);
ALTER TABLE files ADD COLUMN hora_recojo TIME;

-- Migrar datos de vuelta: tomar del primer file_tour de cada file
UPDATE files f
SET 
    turno_tour = ft.turno_tour,
    lugar_recojo = ft.lugar_recojo,
    hora_recojo = ft.hora_recojo
FROM (
    SELECT DISTINCT ON (id_file) 
        id_file, turno_tour, lugar_recojo, hora_recojo
    FROM file_tours
    ORDER BY id_file, orden ASC
) ft
WHERE f.id = ft.id_file;

-- =======================================================================
-- PARTE 3: Eliminar campos de file_tours
-- =======================================================================

ALTER TABLE file_tours DROP COLUMN IF EXISTS turno_tour;
ALTER TABLE file_tours DROP COLUMN IF EXISTS lugar_recojo;
ALTER TABLE file_tours DROP COLUMN IF EXISTS hora_recojo;

-- =======================================================================
-- PARTE 4: Recrear la vista original con f.*
-- =======================================================================

CREATE OR REPLACE VIEW files_with_primary_tour AS
SELECT 
    f.*,
    ft.id_tour AS primary_tour_id,
    t.nombre AS primary_tour_nombre,
    t.precio_base AS primary_tour_precio
FROM files f
LEFT JOIN LATERAL (
    SELECT ft2.id_tour 
    FROM file_tours ft2 
    WHERE ft2.id_file = f.id 
    ORDER BY ft2.orden ASC 
    LIMIT 1
) ft ON true
LEFT JOIN tours t ON t.id = ft.id_tour;

COMMENT ON VIEW files_with_primary_tour IS 'Vista de files con su tour principal (orden=1) para compatibilidad';

-- =======================================================================
-- PARTE 5: Revertir cambios en file_pasajeros
-- =======================================================================

-- 5.1) Eliminar índice parcial
DROP INDEX IF EXISTS idx_file_pasajeros_unique_persona;

-- 5.2) Eliminar campo edad
ALTER TABLE file_pasajeros DROP COLUMN IF EXISTS edad;

-- 5.3) Eliminar pasajeros sin id_persona (no pueden existir en el schema anterior)
DELETE FROM file_pasajeros WHERE id_persona IS NULL;

-- 5.4) Restaurar NOT NULL en id_persona
ALTER TABLE file_pasajeros ALTER COLUMN id_persona SET NOT NULL;

-- 5.5) Eliminar la FK actual
ALTER TABLE file_pasajeros DROP CONSTRAINT IF EXISTS file_pasajeros_id_persona_fkey;

-- 5.6) Recrear la FK original con CASCADE
ALTER TABLE file_pasajeros 
    ADD CONSTRAINT file_pasajeros_id_persona_fkey 
    FOREIGN KEY (id_persona) REFERENCES personas(id) ON DELETE CASCADE;

-- 5.7) Restaurar constraint UNIQUE original
ALTER TABLE file_pasajeros 
    ADD CONSTRAINT uq_file_pasajeros UNIQUE (id_file, id_persona);

