-- ========================================================================
-- MIGRACIÓN: Multiselección de Tours para Files
-- 
-- PROBLEMA: files.id_tour solo permite UN tour por file
-- SOLUCIÓN: Crear tabla file_tours (relación N:M) para permitir múltiples tours
-- ========================================================================

-- 1) Crear tabla de relación file_tours (similar a file_guias, file_vehiculos, etc.)
CREATE TABLE file_tours (
    id SERIAL PRIMARY KEY,
    id_file INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    id_tour INTEGER NOT NULL REFERENCES tours(id) ON DELETE RESTRICT,
    
    -- Orden del tour dentro del file (para tours que se hacen en secuencia)
    orden INTEGER NOT NULL DEFAULT 1,
    
    -- Precio específico para este tour en este file (puede diferir del precio_base)
    precio_aplicado DECIMAL(10,2),
    
    -- Notas específicas para este tour en el contexto del file
    notas TEXT,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    -- Un tour no puede estar duplicado en el mismo file
    CONSTRAINT uq_file_tours UNIQUE (id_file, id_tour)
);

-- Índices para búsquedas
CREATE INDEX idx_file_tours_id_file ON file_tours(id_file);
CREATE INDEX idx_file_tours_id_tour ON file_tours(id_tour);
CREATE INDEX idx_file_tours_orden ON file_tours(id_file, orden);

-- Comentarios de documentación
COMMENT ON TABLE file_tours IS 'Relación N:M entre files y tours - permite múltiples tours por file';
COMMENT ON COLUMN file_tours.orden IS 'Orden secuencial del tour dentro del file (1, 2, 3...)';
COMMENT ON COLUMN file_tours.precio_aplicado IS 'Precio específico aplicado, puede diferir del precio_base del tour';

-- 2) Migrar datos existentes de files.id_tour a file_tours
INSERT INTO file_tours (id_file, id_tour, orden, precio_aplicado, created_at, created_by)
SELECT 
    f.id,
    f.id_tour,
    1,  -- orden inicial
    t.precio_base,  -- usar el precio base del tour
    f.created_at,
    f.created_by
FROM files f
JOIN tours t ON t.id = f.id_tour
WHERE f.id_tour IS NOT NULL;

-- 3) Eliminar la columna id_tour de files (ya no es necesaria)
ALTER TABLE files DROP COLUMN id_tour;

-- 4) Crear una vista auxiliar para obtener el "tour principal" de un file
-- (útil para compatibilidad con código existente)
CREATE OR REPLACE VIEW files_with_primary_tour AS
SELECT 
    f.*,
    ft.id_tour AS primary_tour_id,
    t.nombre AS primary_tour_nombre,
    t.precio_base AS primary_tour_precio
FROM files f
LEFT JOIN LATERAL (
    SELECT ft2.id_tour 
    FROM file_tours ft2 
    WHERE ft2.id_file = f.id 
    ORDER BY ft2.orden ASC 
    LIMIT 1
) ft ON true
LEFT JOIN tours t ON t.id = ft.id_tour;

COMMENT ON VIEW files_with_primary_tour IS 'Vista de files con su tour principal (orden=1) para compatibilidad';
