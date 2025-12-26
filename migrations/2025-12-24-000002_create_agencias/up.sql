-- Migration: Create agencias table
-- Agencias de turismo (tour operators)

CREATE TABLE agencias (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre VARCHAR(200) NOT NULL,
    ruc VARCHAR(11) NOT NULL UNIQUE,
    telefono VARCHAR(20),
    correo VARCHAR(255),
    direccion TEXT,
    paleta_colores JSONB DEFAULT '{}',  -- {"primary": "#...", "secondary": "#..."}
    media JSONB DEFAULT '{}',            -- {"logo": "url", "banner": "url", "images": [...]}
    encargado UUID REFERENCES personas(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT chk_agencias_ruc_format CHECK (ruc ~ '^[0-9]{11}$'),
    CONSTRAINT chk_agencias_correo_format CHECK (
        correo IS NULL OR correo ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'
    )
);

-- Indexes
CREATE INDEX idx_agencias_nombre ON agencias(nombre);
CREATE INDEX idx_agencias_ruc ON agencias(ruc);
CREATE INDEX idx_agencias_encargado ON agencias(encargado);
CREATE INDEX idx_agencias_is_active ON agencias(is_active);

-- Trigger
CREATE TRIGGER update_agencias_updated_at
    BEFORE UPDATE ON agencias
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
