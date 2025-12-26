-- ========================================================================
-- TABLA ENTRADAS
-- Entradas/tickets a lugares turísticos
-- ========================================================================

CREATE TABLE entradas (
    id SERIAL PRIMARY KEY,
    nombre VARCHAR(200) NOT NULL,
    precio DECIMAL(10,2) NOT NULL DEFAULT 0,
    ruta VARCHAR(200),
    tipo VARCHAR(50) NOT NULL DEFAULT 'general',
    descripcion TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    updated_by INTEGER REFERENCES users(id) ON DELETE SET NULL
);

-- Índices
CREATE INDEX idx_entradas_nombre ON entradas(nombre);
CREATE INDEX idx_entradas_tipo ON entradas(tipo);
CREATE INDEX idx_entradas_is_active ON entradas(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_entradas_precio ON entradas(precio);

-- Trigger para updated_at
CREATE TRIGGER update_entradas_updated_at
    BEFORE UPDATE ON entradas
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
