-- ========================================================================
-- TABLA RESTAURANTES
-- Restaurantes asociados a tours
-- ========================================================================

CREATE TABLE restaurantes (
    id SERIAL PRIMARY KEY,
    nombre VARCHAR(200) NOT NULL,
    direccion TEXT NOT NULL,
    telefono VARCHAR(20),
    correo VARCHAR(255),
    tipo_atencion JSONB DEFAULT '["almuerzo"]',
    precio_promedio DECIMAL(10,2) DEFAULT 0,
    capacidad INTEGER,
    horario JSONB DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    updated_by INTEGER REFERENCES users(id) ON DELETE SET NULL
);

-- Índices
CREATE INDEX idx_restaurantes_nombre ON restaurantes(nombre);
CREATE INDEX idx_restaurantes_is_active ON restaurantes(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_restaurantes_precio_promedio ON restaurantes(precio_promedio);

-- Trigger para updated_at
CREATE TRIGGER update_restaurantes_updated_at
    BEFORE UPDATE ON restaurantes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
