-- ========================================================================
-- TABLA AGENCIAS
-- Agencias de turismo (tour operators)
-- ========================================================================

CREATE TABLE agencias (
    id SERIAL PRIMARY KEY,
    nombre VARCHAR(200) NOT NULL,
    ruc VARCHAR(11) NOT NULL UNIQUE,
    telefono VARCHAR(20),
    correo VARCHAR(255),
    direccion TEXT,
    paleta_colores JSONB DEFAULT '{}',
    media JSONB DEFAULT '{}',
    encargado INTEGER REFERENCES personas(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    updated_by INTEGER REFERENCES users(id) ON DELETE SET NULL
);

-- Índices
CREATE UNIQUE INDEX idx_agencias_ruc ON agencias(ruc);
CREATE INDEX idx_agencias_nombre ON agencias(nombre);
CREATE INDEX idx_agencias_is_active ON agencias(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_agencias_encargado ON agencias(encargado) WHERE encargado IS NOT NULL;

-- Trigger para updated_at
CREATE TRIGGER update_agencias_updated_at
    BEFORE UPDATE ON agencias
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
