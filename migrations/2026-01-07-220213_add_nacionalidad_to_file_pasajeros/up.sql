-- Agregar nacionalidad a file_pasajeros
ALTER TABLE file_pasajeros ADD COLUMN nacionalidad VARCHAR(60);

-- Comentario descriptivo
COMMENT ON COLUMN file_pasajeros.nacionalidad IS 'Nacionalidad del pasajero para este file específico';
