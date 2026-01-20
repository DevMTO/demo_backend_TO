-- Revertir migración: Restaurar estructura original con id_file

-- 1) file_restaurantes: restaurar id_file y dia
ALTER TABLE file_restaurantes ADD COLUMN id_file INTEGER;
ALTER TABLE file_restaurantes ADD COLUMN dia INTEGER DEFAULT 1;

-- Migrar datos de vuelta: obtener id_file desde file_tours
UPDATE file_restaurantes fr
SET 
    id_file = ft.id_file,
    dia = ft.orden
FROM file_tours ft
WHERE ft.id = fr.id_file_tour;

-- Hacer id_file NOT NULL y agregar FK
ALTER TABLE file_restaurantes ALTER COLUMN id_file SET NOT NULL;
ALTER TABLE file_restaurantes 
    ADD CONSTRAINT fk_file_restaurantes_file 
    FOREIGN KEY (id_file) REFERENCES files(id) ON DELETE CASCADE;

-- Eliminar columna id_file_tour
ALTER TABLE file_restaurantes DROP COLUMN id_file_tour;

-- Restaurar índice original
DROP INDEX IF EXISTS idx_file_restaurantes_id_file_tour;
CREATE INDEX idx_file_restaurantes_id_file ON file_restaurantes(id_file);


-- 2) file_entradas: restaurar id_file
ALTER TABLE file_entradas ADD COLUMN id_file INTEGER;

-- Migrar datos de vuelta
UPDATE file_entradas fe
SET id_file = ft.id_file
FROM file_tours ft
WHERE ft.id = fe.id_file_tour;

-- Hacer id_file NOT NULL y agregar FK
ALTER TABLE file_entradas ALTER COLUMN id_file SET NOT NULL;
ALTER TABLE file_entradas 
    ADD CONSTRAINT fk_file_entradas_file 
    FOREIGN KEY (id_file) REFERENCES files(id) ON DELETE CASCADE;

-- Eliminar columna id_file_tour
ALTER TABLE file_entradas DROP CONSTRAINT IF EXISTS uq_file_tour_entradas;
ALTER TABLE file_entradas DROP COLUMN id_file_tour;

-- Restaurar constraint y índice original
ALTER TABLE file_entradas ADD CONSTRAINT uq_file_entradas UNIQUE (id_file, id_entrada);
DROP INDEX IF EXISTS idx_file_entradas_id_file_tour;
CREATE INDEX idx_file_entradas_id_file ON file_entradas(id_file);

