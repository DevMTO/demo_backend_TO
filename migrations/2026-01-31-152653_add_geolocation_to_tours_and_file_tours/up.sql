-- ========================================================================
-- MIGRACIÓN: Agregar geolocalización a tours y file_tours
-- 
-- Esta migración agrega campos de coordenadas geográficas usando JSONB
-- para facilitar la integración con Leaflet en el frontend.
--
-- Formato JSONB para coordenadas:
-- { "lat": -13.5250, "lng": -71.9653, "zoom": 15, "label": "Nombre lugar" }
--
-- Formato JSONB para ruta:
-- [{ "lat": -13.5250, "lng": -71.9653 }, { "lat": -13.3, "lng": -72.1 }, ...]
-- ========================================================================

-- =======================================================================
-- PARTE 1: Modificar tabla tours
-- =======================================================================

-- 1.1) Agregar campos de geolocalización
ALTER TABLE tours ADD COLUMN geo_inicio JSONB;
ALTER TABLE tours ADD COLUMN geo_fin JSONB;
ALTER TABLE tours ADD COLUMN geo_ruta JSONB;

-- 1.2) Hacer lugar_inicio y lugar_fin nullable (ahora son opcionales)
ALTER TABLE tours ALTER COLUMN lugar_inicio DROP NOT NULL;
ALTER TABLE tours ALTER COLUMN lugar_fin DROP NOT NULL;

-- 1.3) Crear índices GIN para búsquedas eficientes en JSONB
CREATE INDEX idx_tours_geo_inicio ON tours USING GIN(geo_inicio jsonb_path_ops);
CREATE INDEX idx_tours_geo_fin ON tours USING GIN(geo_fin jsonb_path_ops);

-- 1.4) Comentarios descriptivos
COMMENT ON COLUMN tours.geo_inicio IS 'Coordenadas del punto de inicio: {"lat": float, "lng": float, "zoom": int, "label": string}';
COMMENT ON COLUMN tours.geo_fin IS 'Coordenadas del punto de fin: {"lat": float, "lng": float, "zoom": int, "label": string}';
COMMENT ON COLUMN tours.geo_ruta IS 'Array de puntos para la ruta: [{"lat": float, "lng": float}, ...]';
COMMENT ON COLUMN tours.lugar_inicio IS 'Nombre descriptivo del lugar de inicio (ahora opcional)';
COMMENT ON COLUMN tours.lugar_fin IS 'Nombre descriptivo del lugar de fin (ahora opcional)';

-- =======================================================================
-- PARTE 2: Modificar tabla file_tours
-- =======================================================================

-- 2.1) Agregar campo de geolocalización para punto de recojo
ALTER TABLE file_tours ADD COLUMN geo_recojo JSONB;

-- 2.2) Crear índice GIN
CREATE INDEX idx_file_tours_geo_recojo ON file_tours USING GIN(geo_recojo jsonb_path_ops);

-- 2.3) Comentario descriptivo
COMMENT ON COLUMN file_tours.geo_recojo IS 'Coordenadas del punto de recojo: {"lat": float, "lng": float, "zoom": int, "label": string}';

-- =======================================================================
-- PARTE 3: Función helper para validar formato de coordenadas (opcional)
-- =======================================================================

-- Función para validar que un JSONB tiene formato de coordenadas válido
CREATE OR REPLACE FUNCTION is_valid_geo_point(geo JSONB) RETURNS BOOLEAN AS $$
DECLARE
    lat_val FLOAT;
    lng_val FLOAT;
BEGIN
    IF geo IS NULL THEN
        RETURN TRUE;  -- NULL es válido (campo opcional)
    END IF;
    
    -- Verificar que tiene lat y lng como números
    IF NOT (
        geo ? 'lat' AND 
        geo ? 'lng' AND
        jsonb_typeof(geo->'lat') = 'number' AND
        jsonb_typeof(geo->'lng') = 'number'
    ) THEN
        RETURN FALSE;
    END IF;
    
    -- Verificar rangos válidos
    lat_val := (geo->>'lat')::FLOAT;
    lng_val := (geo->>'lng')::FLOAT;
    
    -- Latitud: -90 a 90, Longitud: -180 a 180
    IF lat_val < -90 OR lat_val > 90 THEN
        RETURN FALSE;
    END IF;
    IF lng_val < -180 OR lng_val > 180 THEN
        RETURN FALSE;
    END IF;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Comentario para la función
COMMENT ON FUNCTION is_valid_geo_point(JSONB) IS 'Valida que un JSONB tenga formato de coordenadas válido';

-- =======================================================================
-- PARTE 4: Constraints CHECK para validación (opcional pero recomendado)
-- =======================================================================

-- Validar formato de geo_inicio
ALTER TABLE tours ADD CONSTRAINT chk_tours_geo_inicio_valid 
    CHECK (is_valid_geo_point(geo_inicio));

-- Validar formato de geo_fin
ALTER TABLE tours ADD CONSTRAINT chk_tours_geo_fin_valid 
    CHECK (is_valid_geo_point(geo_fin));

-- Validar formato de geo_recojo
ALTER TABLE file_tours ADD CONSTRAINT chk_file_tours_geo_recojo_valid 
    CHECK (is_valid_geo_point(geo_recojo));
