-- Migration: Create entradas table
-- Entradas/tickets a lugares turísticos

CREATE TABLE entradas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre VARCHAR(200) NOT NULL,
    precio DECIMAL(10,2) NOT NULL DEFAULT 0,
    ruta VARCHAR(200),  -- Ruta/lugar del ticket
    tipo VARCHAR(50) NOT NULL DEFAULT 'general',  -- "general", "turista_nacional", "turista_extranjero", "estudiante"
    descripcion TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT chk_entradas_precio CHECK (precio >= 0),
    CONSTRAINT chk_entradas_tipo CHECK (
        tipo IN ('general', 'turista_nacional', 'turista_extranjero', 'estudiante', 'menor', 'adulto_mayor')
    )
);

-- Indexes
CREATE INDEX idx_entradas_nombre ON entradas(nombre);
CREATE INDEX idx_entradas_tipo ON entradas(tipo);
CREATE INDEX idx_entradas_is_active ON entradas(is_active);
CREATE INDEX idx_entradas_precio ON entradas(precio);

-- Trigger
CREATE TRIGGER update_entradas_updated_at
    BEFORE UPDATE ON entradas
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
