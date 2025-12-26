-- Migration: Create tours table
-- Tours/paquetes turísticos

CREATE TABLE tours (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_agencia UUID NOT NULL REFERENCES agencias(id) ON DELETE CASCADE,
    nombre VARCHAR(200) NOT NULL,
    lugar_inicio VARCHAR(200) NOT NULL,
    lugar_fin VARCHAR(200) NOT NULL,
    hora_inicio TIMESTAMPTZ,
    hora_fin TIMESTAMPTZ,
    detalles JSONB DEFAULT '{}',      -- {"incluye": [...], "no_incluye": [...], "recomendaciones": [...]}
    itinerario JSONB DEFAULT '[]',    -- [{"dia": 1, "actividades": [...], "hora": "...", "lugar": "..."}]
    precio DECIMAL(10,2) NOT NULL DEFAULT 0,
    duracion_dias INTEGER DEFAULT 1,
    max_personas INTEGER,
    media JSONB DEFAULT '{}',         -- {"imagenes": [...], "videos": [...]}
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT chk_tours_precio CHECK (precio >= 0),
    CONSTRAINT chk_tours_duracion CHECK (duracion_dias > 0)
);

-- Indexes
CREATE INDEX idx_tours_id_agencia ON tours(id_agencia);
CREATE INDEX idx_tours_nombre ON tours(nombre);
CREATE INDEX idx_tours_lugar_inicio ON tours(lugar_inicio);
CREATE INDEX idx_tours_lugar_fin ON tours(lugar_fin);
CREATE INDEX idx_tours_is_active ON tours(is_active);
CREATE INDEX idx_tours_precio ON tours(precio);

-- Trigger
CREATE TRIGGER update_tours_updated_at
    BEFORE UPDATE ON tours
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
