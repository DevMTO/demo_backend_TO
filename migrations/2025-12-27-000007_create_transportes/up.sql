-- ========================================================================
-- TABLA TRANSPORTES
-- Empresas de transporte terrestre
-- ========================================================================

CREATE TABLE transportes (
    id SERIAL PRIMARY KEY,
    nombre VARCHAR(200) NOT NULL,
    ruc VARCHAR(11) NOT NULL UNIQUE,
    telefono VARCHAR(20),
    correo VARCHAR(255),
    direccion TEXT,
    encargado INTEGER REFERENCES personas(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    updated_by INTEGER REFERENCES users(id) ON DELETE SET NULL
);

-- Índices
CREATE UNIQUE INDEX idx_transportes_ruc ON transportes(ruc);
CREATE INDEX idx_transportes_nombre ON transportes(nombre);
CREATE INDEX idx_transportes_is_active ON transportes(is_active) WHERE is_active = TRUE;

-- Trigger para updated_at
CREATE TRIGGER update_transportes_updated_at
    BEFORE UPDATE ON transportes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
