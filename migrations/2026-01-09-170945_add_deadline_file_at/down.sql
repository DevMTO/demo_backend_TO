-- Revertir migración de deadline_confirmacion

-- Eliminar función
DROP FUNCTION IF EXISTS check_and_expire_files();

-- Eliminar índice
DROP INDEX IF EXISTS idx_files_deadline_confirmacion;

-- Revertir status de 'reservado' a 'pendiente' (si se quiere mantener consistencia)
UPDATE files SET status = 'pendiente' WHERE status = 'reservado';

-- Eliminar columna
ALTER TABLE files DROP COLUMN IF EXISTS deadline_confirmacion;

-- Nota: Los files que quedaron en otros estados (confirmado, asignado, etc.) 
-- deberán ser manejados manualmente si es necesario
