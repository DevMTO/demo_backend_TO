-- ========================================================================
-- TABLA TOURS
-- Tours/paquetes turísticos disponibles
-- La relación tour-agencia se maneja a través de FILES
-- ========================================================================

CREATE TABLE tours (
    id SERIAL PRIMARY KEY,
    nombre VARCHAR(200) NOT NULL,
    lugar_inicio VARCHAR(200) NOT NULL,
    lugar_fin VARCHAR(200) NOT NULL,
    hora_inicio TIME,
    hora_fin TIME,
    detalles JSONB DEFAULT '{}',
    itinerario JSONB DEFAULT '[]',
    precio_base DECIMAL(10,2) NOT NULL DEFAULT 0,
    duracion_dias INTEGER DEFAULT 1,
    max_personas INTEGER,
    media JSONB DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    updated_by INTEGER REFERENCES users(id) ON DELETE SET NULL
);

-- Índices para búsquedas y filtros
CREATE INDEX idx_tours_nombre ON tours(nombre);
CREATE INDEX idx_tours_is_active ON tours(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_tours_precio_base ON tours(precio_base);
CREATE INDEX idx_tours_duracion_dias ON tours(duracion_dias);
CREATE INDEX idx_tours_lugares ON tours(lugar_inicio, lugar_fin);

-- Trigger para updated_at
CREATE TRIGGER update_tours_updated_at
    BEFORE UPDATE ON tours
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
