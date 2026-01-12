-- ========================================================================
-- MIGRACIÓN: Agregar deadline de confirmación a files
-- Permite definir una fecha límite para confirmar un file reservado
-- Si no se confirma antes del deadline, el status cambia a "anulado"
-- ========================================================================

-- Agregar columna deadline_confirmacion a files
ALTER TABLE files 
ADD COLUMN deadline_confirmacion TIMESTAMPTZ NULL;

-- Agregar índice para buscar files con deadline próximo
CREATE INDEX idx_files_deadline_confirmacion 
ON files(deadline_confirmacion) 
WHERE deadline_confirmacion IS NOT NULL 
  AND status = 'reservado';

-- ========================================================================
-- ESTADOS VÁLIDOS DE FILES (Flujo de trabajo):
-- ========================================================================
-- - reservado: Estado inicial al crear el file (creado por agencia)
-- - confirmado: File confirmado antes del deadline por el admin/operador
-- - asignado: File con recursos asignados (guías, vehículos, etc.)
-- - en_curso: El tour está en progreso
-- - completado: El tour finalizó exitosamente
-- - anulado: No se confirmó a tiempo o fue cancelado
-- ========================================================================

-- Actualizar files existentes con status 'pendiente' a 'reservado'
UPDATE files SET status = 'reservado' WHERE status = 'pendiente';

COMMENT ON COLUMN files.deadline_confirmacion IS 
'Fecha límite para confirmar un file en estado reservado. Si se supera sin confirmar, el status pasa a anulado automáticamente.';

COMMENT ON COLUMN files.status IS 
'Estados del file: reservado (inicial), confirmado, asignado, en_curso, completado, anulado';

-- Función para verificar y actualizar files expirados
-- Esta función puede ser llamada periódicamente por un job o al consultar files
CREATE OR REPLACE FUNCTION check_and_expire_files()
RETURNS INTEGER AS $$
DECLARE
    affected_count INTEGER;
BEGIN
    UPDATE files 
    SET status = 'anulado',
        updated_at = NOW()
    WHERE status = 'reservado'
      AND deadline_confirmacion IS NOT NULL
      AND deadline_confirmacion < NOW();
    
    GET DIAGNOSTICS affected_count = ROW_COUNT;
    RETURN affected_count;
END;
$$ LANGUAGE plpgsql;
