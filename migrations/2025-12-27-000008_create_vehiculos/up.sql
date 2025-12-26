-- ========================================================================
-- TABLA VEHICULOS
-- Vehículos de las empresas de transporte
-- ========================================================================

CREATE TABLE vehiculos (
    id SERIAL PRIMARY KEY,
    id_transporte INTEGER NOT NULL REFERENCES transportes(id) ON DELETE CASCADE,
    nombre VARCHAR(100) NOT NULL,
    modelo VARCHAR(100),
    placa VARCHAR(10) NOT NULL UNIQUE,
    capacidad INTEGER NOT NULL DEFAULT 1,
    status VARCHAR(20) NOT NULL DEFAULT 'disponible',
    media JSONB DEFAULT '{}',
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    updated_by INTEGER REFERENCES users(id) ON DELETE SET NULL
);

-- Índices
CREATE UNIQUE INDEX idx_vehiculos_placa ON vehiculos(placa);
CREATE INDEX idx_vehiculos_id_transporte ON vehiculos(id_transporte);
CREATE INDEX idx_vehiculos_status ON vehiculos(status);
CREATE INDEX idx_vehiculos_disponibles ON vehiculos(status, capacidad) WHERE status = 'disponible';

-- Trigger para updated_at
CREATE TRIGGER update_vehiculos_updated_at
    BEFORE UPDATE ON vehiculos
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
