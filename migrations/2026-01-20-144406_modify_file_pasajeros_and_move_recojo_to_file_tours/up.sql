-- ========================================================================
-- MIGRACIÓN: Modificar file_pasajeros, mover campos de recojo a file_tours
--            y agregar tiene_restaurante a tours
-- 
-- CAMBIOS EN file_pasajeros:
--   1. Hacer id_persona nullable (pasajeros sin registro en personas)
--   2. Agregar campo edad
--   3. Eliminar constraint UNIQUE (ya no aplica si id_persona puede ser NULL)
--
-- CAMBIOS EN file_tours:
--   1. Agregar turno_tour (mover desde files)
--   2. Agregar lugar_recojo (mover desde files)
--   3. Agregar hora_recojo (mover desde files)
--
-- CAMBIOS EN files:
--   1. Eliminar turno_tour, lugar_recojo, hora_recojo (ya están en file_tours)
--
-- CAMBIOS EN tours:
--   1. Agregar tiene_restaurante (boolean default false)
-- ========================================================================

-- =======================================================================
-- PARTE 1: Modificar file_pasajeros
-- =======================================================================

-- 1.1) Eliminar el constraint UNIQUE actual
ALTER TABLE file_pasajeros DROP CONSTRAINT IF EXISTS uq_file_pasajeros;

-- 1.2) Hacer id_persona nullable (ya no es requerido)
ALTER TABLE file_pasajeros ALTER COLUMN id_persona DROP NOT NULL;

-- 1.3) Eliminar la FK actual que requiere id_persona
ALTER TABLE file_pasajeros DROP CONSTRAINT IF EXISTS file_pasajeros_id_persona_fkey;

-- 1.4) Recrear la FK pero permitiendo NULL
ALTER TABLE file_pasajeros 
    ADD CONSTRAINT file_pasajeros_id_persona_fkey 
    FOREIGN KEY (id_persona) REFERENCES personas(id) ON DELETE SET NULL;

-- 1.5) Agregar campo edad
ALTER TABLE file_pasajeros ADD COLUMN edad INTEGER;

COMMENT ON COLUMN file_pasajeros.id_persona IS 'Referencia opcional a persona registrada (puede ser NULL para pasajeros anónimos)';
COMMENT ON COLUMN file_pasajeros.edad IS 'Edad del pasajero al momento del viaje';
COMMENT ON COLUMN file_pasajeros.nacionalidad IS 'Nacionalidad del pasajero para este file';
COMMENT ON COLUMN file_pasajeros.tipo_pasajero IS 'Tipo: adulto, nino, infante, tercera_edad';

-- =======================================================================
-- PARTE 2: Agregar campos de recojo a file_tours
-- =======================================================================

-- 2.1) Agregar nuevos campos a file_tours
ALTER TABLE file_tours ADD COLUMN turno_tour VARCHAR(30);
ALTER TABLE file_tours ADD COLUMN lugar_recojo VARCHAR(200);
ALTER TABLE file_tours ADD COLUMN hora_recojo TIME;

COMMENT ON COLUMN file_tours.turno_tour IS 'Turno del tour: manana, tarde, noche';
COMMENT ON COLUMN file_tours.lugar_recojo IS 'Lugar de recojo para este tour específico';
COMMENT ON COLUMN file_tours.hora_recojo IS 'Hora de recojo para este tour específico';

-- 2.2) Migrar datos existentes de files a file_tours
-- Cada file_tour hereda los valores de su file padre
UPDATE file_tours ft
SET 
    turno_tour = f.turno_tour,
    lugar_recojo = f.lugar_recojo,
    hora_recojo = f.hora_recojo
FROM files f
WHERE ft.id_file = f.id;

-- =======================================================================
-- PARTE 3: Eliminar vista que depende de files.*
-- =======================================================================

DROP VIEW IF EXISTS files_with_primary_tour CASCADE;

-- =======================================================================
-- PARTE 4: Eliminar campos de files (ya migrados a file_tours)
-- =======================================================================

ALTER TABLE files DROP COLUMN IF EXISTS turno_tour;
ALTER TABLE files DROP COLUMN IF EXISTS lugar_recojo;
ALTER TABLE files DROP COLUMN IF EXISTS hora_recojo;

-- =======================================================================
-- PARTE 5: Agregar tiene_restaurante a tours
-- =======================================================================

ALTER TABLE tours ADD COLUMN tiene_restaurante BOOLEAN NOT NULL DEFAULT FALSE;

COMMENT ON COLUMN tours.tiene_restaurante IS 'Indica si el tour incluye restaurante en su itinerario';

-- =======================================================================
-- PARTE 6: Crear índice parcial para uniqueness solo cuando id_persona existe
-- =======================================================================

-- Si hay id_persona, debe ser único por file
CREATE UNIQUE INDEX idx_file_pasajeros_unique_persona 
    ON file_pasajeros(id_file, id_persona) 
    WHERE id_persona IS NOT NULL;

