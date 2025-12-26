-- Migration: Create guias table
-- Guías turísticos

CREATE TABLE guias (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_persona UUID NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    nro_carnet VARCHAR(30) NOT NULL,
    idiomas JSONB DEFAULT '["Español"]',  -- ["Español", "Inglés", "Francés"]
    especialidades JSONB DEFAULT '[]',     -- ["City tours", "Aventura", "Cultural"]
    status VARCHAR(20) NOT NULL DEFAULT 'disponible',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT chk_guias_status CHECK (
        status IN ('disponible', 'en_servicio', 'inactivo', 'suspendido')
    ),
    -- Una persona solo puede ser guía una vez
    CONSTRAINT uq_guias_persona UNIQUE (id_persona)
);

-- Indexes
CREATE INDEX idx_guias_id_persona ON guias(id_persona);
CREATE INDEX idx_guias_nro_carnet ON guias(nro_carnet);
CREATE INDEX idx_guias_status ON guias(status);

-- Trigger
CREATE TRIGGER update_guias_updated_at
    BEFORE UPDATE ON guias
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
