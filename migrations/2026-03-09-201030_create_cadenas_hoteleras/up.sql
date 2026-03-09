-- ========================================================================
-- TABLA CADENAS_HOTELERAS (Proveedores de hoteles / Cadenas)
-- Similar a agencias pero orientado a cadenas de hoteles
-- ========================================================================

CREATE TABLE cadenas_hoteleras (
    id SERIAL PRIMARY KEY,
    nombre VARCHAR(200) NOT NULL,
    telefono VARCHAR(20),
    correo VARCHAR(255),
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
CREATE INDEX idx_cadenas_hoteleras_nombre ON cadenas_hoteleras(nombre);
CREATE INDEX idx_cadenas_hoteleras_is_active ON cadenas_hoteleras(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_cadenas_hoteleras_encargado ON cadenas_hoteleras(encargado) WHERE encargado IS NOT NULL;

-- Trigger para updated_at
CREATE TRIGGER update_cadenas_hoteleras_updated_at
    BEFORE UPDATE ON cadenas_hoteleras
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
