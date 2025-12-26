-- ========================================================================
-- TABLA GUIAS
-- Guías turísticos certificados
-- ========================================================================

CREATE TABLE guias (
    id SERIAL PRIMARY KEY,
    id_persona INTEGER NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    nro_carnet VARCHAR(30) NOT NULL,
    idiomas JSONB DEFAULT '["Español"]',
    especialidades JSONB DEFAULT '[]',
    status VARCHAR(20) NOT NULL DEFAULT 'disponible',
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    updated_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    -- Una persona solo puede ser guía una vez
    CONSTRAINT uq_guias_persona UNIQUE (id_persona)
);

-- Índices
CREATE INDEX idx_guias_id_persona ON guias(id_persona);
CREATE INDEX idx_guias_nro_carnet ON guias(nro_carnet);
CREATE INDEX idx_guias_status ON guias(status);
CREATE INDEX idx_guias_disponibles ON guias(status) WHERE status = 'disponible';

-- Trigger para updated_at
CREATE TRIGGER update_guias_updated_at
    BEFORE UPDATE ON guias
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
