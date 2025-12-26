-- Migration: Create transportes table
-- Empresas de transporte

CREATE TABLE transportes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre VARCHAR(200) NOT NULL,
    ruc VARCHAR(11) NOT NULL UNIQUE,
    telefono VARCHAR(20),
    correo VARCHAR(255),
    direccion TEXT,
    encargado UUID REFERENCES personas(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT chk_transportes_ruc_format CHECK (ruc ~ '^[0-9]{11}$'),
    CONSTRAINT chk_transportes_correo_format CHECK (
        correo IS NULL OR correo ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'
    )
);

-- Indexes
CREATE INDEX idx_transportes_nombre ON transportes(nombre);
CREATE INDEX idx_transportes_ruc ON transportes(ruc);
CREATE INDEX idx_transportes_encargado ON transportes(encargado);
CREATE INDEX idx_transportes_is_active ON transportes(is_active);

-- Trigger
CREATE TRIGGER update_transportes_updated_at
    BEFORE UPDATE ON transportes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
