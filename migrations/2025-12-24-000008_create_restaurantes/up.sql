-- Migration: Create restaurantes table
-- Restaurantes asociados a tours

CREATE TABLE restaurantes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre VARCHAR(200) NOT NULL,
    direccion TEXT NOT NULL,
    telefono VARCHAR(20),
    correo VARCHAR(255),
    tipo_atencion JSONB DEFAULT '["almuerzo"]',  -- ["desayuno", "almuerzo", "cena", "snacks"]
    precio_promedio DECIMAL(10,2) DEFAULT 0,
    capacidad INTEGER,
    horario JSONB DEFAULT '{}',  -- {"apertura": "08:00", "cierre": "22:00", "dias": [...]}
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT chk_restaurantes_precio CHECK (precio_promedio >= 0),
    CONSTRAINT chk_restaurantes_correo_format CHECK (
        correo IS NULL OR correo ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'
    )
);

-- Indexes
CREATE INDEX idx_restaurantes_nombre ON restaurantes(nombre);
CREATE INDEX idx_restaurantes_is_active ON restaurantes(is_active);
CREATE INDEX idx_restaurantes_precio_promedio ON restaurantes(precio_promedio);

-- Trigger
CREATE TRIGGER update_restaurantes_updated_at
    BEFORE UPDATE ON restaurantes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
