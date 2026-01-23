-- ========================================================================
-- MIGRACIÓN: Agregar status a tablas de relación de files (que aún no lo tienen)
-- 
-- Tablas afectadas:
--   - file_entradas
--   - file_vehiculos
--   - file_restaurantes
--   - file_guias (lógica especial basada en aceptado)
--   - file_pasajeros
--
-- NOTA: files y file_tours YA tienen status, no se modifican
--
-- Status posibles: 'pendiente', 'reservado', 'confirmado', 'en_progreso', 'completado', 'cancelado'
-- Por defecto: 'reservado' (excepto file_guias que depende de 'aceptado')
-- ========================================================================

-- 1) Agregar status a file_entradas
ALTER TABLE file_entradas 
ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'reservado';

COMMENT ON COLUMN file_entradas.status IS 'Estado de la entrada en el file: reservado, confirmado, cancelado';
CREATE INDEX idx_file_entradas_status ON file_entradas(status);

-- 2) Agregar status a file_vehiculos
ALTER TABLE file_vehiculos 
ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'reservado';

COMMENT ON COLUMN file_vehiculos.status IS 'Estado del vehículo en el file: reservado, confirmado, cancelado';
CREATE INDEX idx_file_vehiculos_status ON file_vehiculos(status);

-- 3) Agregar status a file_restaurantes
ALTER TABLE file_restaurantes 
ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'reservado';

COMMENT ON COLUMN file_restaurantes.status IS 'Estado del restaurante en el file: reservado, confirmado, cancelado';
CREATE INDEX idx_file_restaurantes_status ON file_restaurantes(status);

-- 4) Agregar status a file_guias (lógica especial)
-- Si el guía NO ha aceptado -> 'pendiente'
-- Si el guía SÍ ha aceptado -> 'reservado'
ALTER TABLE file_guias 
ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'pendiente';

-- Actualizar status basado en el campo 'aceptado' existente (si existe)
-- Si aceptado = true -> status = 'reservado'
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'file_guias' AND column_name = 'aceptado') THEN
        UPDATE file_guias SET status = 'reservado' WHERE aceptado = true;
    END IF;
END $$;

COMMENT ON COLUMN file_guias.status IS 'Estado del guía en el file: pendiente (no aceptado), reservado (aceptado), confirmado, cancelado';
CREATE INDEX idx_file_guias_status ON file_guias(status);

-- 5) Agregar status a file_pasajeros
ALTER TABLE file_pasajeros 
ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'reservado';

COMMENT ON COLUMN file_pasajeros.status IS 'Estado del pasajero en el file: reservado, confirmado, no_show, cancelado';
CREATE INDEX idx_file_pasajeros_status ON file_pasajeros(status);

