-- Revertir cambios de uq_file_entradas
-- Volver al constraint original que NO considera precio

-- Eliminar el índice único que considera precio
DROP INDEX IF EXISTS uq_file_entradas_tour_precio;

-- Restaurar el constraint único original
ALTER TABLE file_entradas ADD CONSTRAINT uq_file_tour_entradas UNIQUE (id_file_tour, id_entrada);
