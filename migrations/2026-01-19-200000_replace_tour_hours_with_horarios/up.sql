-- ========================================================================
-- MIGRACIÓN: Reemplazar hora_inicio/hora_fin por campo JSONB 'horarios'
-- ========================================================================
-- Esto permite manejar horarios flexibles para tours:
-- 
-- FULL DAY (un solo horario):
--   { "full": { "start": "08:00", "end": "18:00" } }
--
-- HALF DAY solo mañana (morning):
--   { "morning": { "start": "08:00", "end": "12:00" } }
--
-- HALF DAY solo tarde (afternoon):
--   { "afternoon": { "start": "14:00", "end": "18:00" } }
--
-- HALF DAY ambos turnos disponibles:
--   { 
--     "morning": { "start": "08:00", "end": "12:00" },
--     "afternoon": { "start": "14:00", "end": "18:00" }
--   }
-- ========================================================================

-- Paso 1: Agregar la nueva columna horarios
ALTER TABLE tours 
ADD COLUMN horarios JSONB DEFAULT '{}';

-- Paso 2: Migrar datos existentes de hora_inicio/hora_fin a horarios
-- Si tiene hora_inicio y hora_fin, lo guardamos como horario full (único)
UPDATE tours 
SET horarios = jsonb_build_object(
    'full', jsonb_build_object(
        'start', to_char(hora_inicio, 'HH24:MI'),
        'end', to_char(hora_fin, 'HH24:MI')
    )
)
WHERE hora_inicio IS NOT NULL AND hora_fin IS NOT NULL;

-- Paso 3: Eliminar las columnas antiguas
ALTER TABLE tours DROP COLUMN hora_inicio;
ALTER TABLE tours DROP COLUMN hora_fin;

-- Paso 4: Agregar comentario descriptivo
COMMENT ON COLUMN tours.horarios IS 'Horarios del tour en formato JSONB. Estructura: { "morning": {"start": "HH:MM", "end": "HH:MM"}, "afternoon": {...}, "full": {...} }';
