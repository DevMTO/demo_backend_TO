-- ========================================================================
-- TABLA HOTELES (Sucursales/Hoteles de cada Cadena Hotelera)
-- Cada hotel pertenece a una cadena hotelera (proveedor)
-- Funciona similar a agencia: puede crear files, tener gerente, etc.
-- ========================================================================

CREATE TABLE hoteles (
    id SERIAL PRIMARY KEY,
    id_cadena INTEGER NOT NULL REFERENCES cadenas_hoteleras(id) ON DELETE CASCADE,
    nombre VARCHAR(200) NOT NULL,
    categoria VARCHAR(50),
    telefono VARCHAR(20),
    correo VARCHAR(255),
    direccion TEXT,
    ciudad VARCHAR(100),
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
CREATE INDEX idx_hoteles_id_cadena ON hoteles(id_cadena);
CREATE INDEX idx_hoteles_nombre ON hoteles(nombre);
CREATE INDEX idx_hoteles_ciudad ON hoteles(ciudad) WHERE ciudad IS NOT NULL;
CREATE INDEX idx_hoteles_is_active ON hoteles(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_hoteles_encargado ON hoteles(encargado) WHERE encargado IS NOT NULL;

-- Trigger para updated_at
CREATE TRIGGER update_hoteles_updated_at
    BEFORE UPDATE ON hoteles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
