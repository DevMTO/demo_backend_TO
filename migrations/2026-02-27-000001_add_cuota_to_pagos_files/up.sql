-- Agregar columna cuota a pagos_files para indexar los pagos de un file_tour
ALTER TABLE pagos_files ADD COLUMN cuota SMALLINT;
