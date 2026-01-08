-- Revertir: eliminar columna turno_tour de files
ALTER TABLE files DROP COLUMN IF EXISTS turno_tour;