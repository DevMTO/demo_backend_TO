-- Migration: Create conductores table
-- Conductores asociados a empresas de transporte

CREATE TABLE conductores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_persona UUID NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    id_transporte UUID REFERENCES transportes(id) ON DELETE SET NULL,
    nro_brevete VARCHAR(20) NOT NULL,
    tiene_soat BOOLEAN NOT NULL DEFAULT FALSE,
    status VARCHAR(20) NOT NULL DEFAULT 'disponible',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT chk_conductores_status CHECK (
        status IN ('disponible', 'en_servicio', 'inactivo', 'suspendido')
    ),
    -- Un persona solo puede ser conductor una vez
    CONSTRAINT uq_conductores_persona UNIQUE (id_persona)
);

-- Indexes
CREATE INDEX idx_conductores_id_persona ON conductores(id_persona);
CREATE INDEX idx_conductores_id_transporte ON conductores(id_transporte);
CREATE INDEX idx_conductores_nro_brevete ON conductores(nro_brevete);
CREATE INDEX idx_conductores_status ON conductores(status);

-- Trigger
CREATE TRIGGER update_conductores_updated_at
    BEFORE UPDATE ON conductores
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
