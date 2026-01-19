-- ========================================================================
-- ROLLBACK: Restaurar hora_inicio/hora_fin desde horarios
-- ========================================================================

-- Paso 1: Agregar las columnas antiguas
ALTER TABLE tours ADD COLUMN hora_inicio TIME;
ALTER TABLE tours ADD COLUMN hora_fin TIME;

-- Paso 2: Restaurar datos desde horarios
-- Primero intentamos con 'full', luego con 'morning'
UPDATE tours 
SET 
    hora_inicio = CASE 
        WHEN horarios ? 'full' THEN (horarios->'full'->>'start')::TIME
        WHEN horarios ? 'morning' THEN (horarios->'morning'->>'start')::TIME
        ELSE NULL
    END,
    hora_fin = CASE 
        WHEN horarios ? 'full' THEN (horarios->'full'->>'end')::TIME
        WHEN horarios ? 'morning' THEN (horarios->'morning'->>'end')::TIME
        ELSE NULL
    END
WHERE horarios IS NOT NULL AND horarios != '{}';

-- Paso 3: Eliminar la columna horarios
ALTER TABLE tours DROP COLUMN horarios;

-- Paso 4: Quitar comentario (no necesario, la columna se elimina)
