-- ========================================================================
-- TABLA CONDUCTORES
-- Conductores asociados a transportes
-- ========================================================================

CREATE TABLE conductores (
    id SERIAL PRIMARY KEY,
    id_persona INTEGER NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    id_transporte INTEGER REFERENCES transportes(id) ON DELETE SET NULL,
    nro_brevete VARCHAR(20) NOT NULL,
    tiene_soat BOOLEAN NOT NULL DEFAULT FALSE,
    status VARCHAR(20) NOT NULL DEFAULT 'disponible',
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    updated_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    -- Una persona solo puede ser conductor una vez
    CONSTRAINT uq_conductores_persona UNIQUE (id_persona)
);

-- Índices
CREATE INDEX idx_conductores_id_persona ON conductores(id_persona);
CREATE INDEX idx_conductores_id_transporte ON conductores(id_transporte) WHERE id_transporte IS NOT NULL;
CREATE INDEX idx_conductores_status ON conductores(status);
CREATE INDEX idx_conductores_disponibles ON conductores(id_transporte, status) WHERE status = 'disponible';

-- Trigger para updated_at
CREATE TRIGGER update_conductores_updated_at
    BEFORE UPDATE ON conductores
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
