-- ========================================================================
-- ROLLBACK: Revertir multiselección de tours
-- ========================================================================

-- 1) Eliminar la vista
DROP VIEW IF EXISTS files_with_primary_tour;

-- 2) Recrear columna id_tour en files
ALTER TABLE files ADD COLUMN id_tour INTEGER;

-- 3) Migrar datos de vuelta (tomamos el primer tour por orden)
UPDATE files f
SET id_tour = (
    SELECT ft.id_tour 
    FROM file_tours ft 
    WHERE ft.id_file = f.id 
    ORDER BY ft.orden ASC 
    LIMIT 1
);

-- 4) Hacer id_tour NOT NULL y agregar FK
ALTER TABLE files ALTER COLUMN id_tour SET NOT NULL;
ALTER TABLE files ADD CONSTRAINT files_id_tour_fkey FOREIGN KEY (id_tour) REFERENCES tours(id) ON DELETE RESTRICT;

-- 5) Recrear índice
CREATE INDEX IF NOT EXISTS idx_files_id_tour ON files(id_tour);

-- 6) Eliminar tabla file_tours
DROP TABLE IF EXISTS file_tours;
