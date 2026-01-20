-- ========================================================================
-- MIGRACIÓN: Vincular file_restaurantes y file_entradas a file_tours
-- 
-- PROBLEMA: file_restaurantes y file_entradas están vinculados a files.id
-- SOLUCIÓN: Cambiar para que apunten a file_tours.id (cada tour tiene su restaurante/entrada)
-- ========================================================================

-- 1) MODIFICAR file_restaurantes: cambiar id_file por id_file_tour
-- Primero agregar la nueva columna
ALTER TABLE file_restaurantes ADD COLUMN id_file_tour INTEGER REFERENCES file_tours(id) ON DELETE CASCADE;

-- Migrar datos: buscar el file_tour correspondiente por id_file y dia/orden
UPDATE file_restaurantes fr
SET id_file_tour = ft.id
FROM file_tours ft
WHERE ft.id_file = fr.id_file
  AND (ft.orden = fr.dia OR (fr.dia IS NULL AND ft.orden = 1));

-- Hacer la columna obligatoria para nuevos registros (los viejos sin match se eliminarán)
DELETE FROM file_restaurantes WHERE id_file_tour IS NULL;
ALTER TABLE file_restaurantes ALTER COLUMN id_file_tour SET NOT NULL;

-- Eliminar la columna id_file y la columna dia (ya no necesarias, el orden está en file_tours)
ALTER TABLE file_restaurantes DROP COLUMN id_file;
ALTER TABLE file_restaurantes DROP COLUMN dia;

-- Crear índice para la nueva FK
DROP INDEX IF EXISTS idx_file_restaurantes_id_file;
CREATE INDEX idx_file_restaurantes_id_file_tour ON file_restaurantes(id_file_tour);

-- 2) MODIFICAR file_entradas: cambiar id_file por id_file_tour
ALTER TABLE file_entradas ADD COLUMN id_file_tour INTEGER REFERENCES file_tours(id) ON DELETE CASCADE;

-- Migrar datos: asignar al primer tour del file (orden = 1)
UPDATE file_entradas fe
SET id_file_tour = ft.id
FROM file_tours ft
WHERE ft.id_file = fe.id_file
  AND ft.orden = 1;

-- Hacer la columna obligatoria para nuevos registros
DELETE FROM file_entradas WHERE id_file_tour IS NULL;
ALTER TABLE file_entradas ALTER COLUMN id_file_tour SET NOT NULL;

-- Eliminar la columna id_file (ya no necesaria)
ALTER TABLE file_entradas DROP COLUMN id_file;

-- Eliminar constraint único antiguo y crear uno nuevo basado en file_tour
ALTER TABLE file_entradas DROP CONSTRAINT IF EXISTS uq_file_entradas;
ALTER TABLE file_entradas ADD CONSTRAINT uq_file_tour_entradas UNIQUE (id_file_tour, id_entrada);

-- Actualizar índices
DROP INDEX IF EXISTS idx_file_entradas_id_file;
CREATE INDEX idx_file_entradas_id_file_tour ON file_entradas(id_file_tour);

-- Comentarios de documentación
COMMENT ON COLUMN file_restaurantes.id_file_tour IS 'FK al tour específico donde aplica este restaurante';
COMMENT ON COLUMN file_entradas.id_file_tour IS 'FK al tour específico donde aplica esta entrada';

