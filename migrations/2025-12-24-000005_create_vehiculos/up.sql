-- Migration: Create vehiculos table
-- Vehículos de las empresas de transporte

CREATE TABLE vehiculos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_transporte UUID NOT NULL REFERENCES transportes(id) ON DELETE CASCADE,
    nombre VARCHAR(100) NOT NULL,
    modelo VARCHAR(100),
    placa VARCHAR(10) NOT NULL UNIQUE,
    capacidad INTEGER NOT NULL DEFAULT 1,
    status VARCHAR(20) NOT NULL DEFAULT 'disponible',
    media JSONB DEFAULT '{}',  -- {"fotos": [...]}
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT chk_vehiculos_capacidad CHECK (capacidad > 0),
    CONSTRAINT chk_vehiculos_status CHECK (
        status IN ('disponible', 'en_uso', 'mantenimiento', 'fuera_servicio')
    )
);

-- Indexes
CREATE INDEX idx_vehiculos_id_transporte ON vehiculos(id_transporte);
CREATE INDEX idx_vehiculos_placa ON vehiculos(placa);
CREATE INDEX idx_vehiculos_status ON vehiculos(status);
CREATE INDEX idx_vehiculos_capacidad ON vehiculos(capacidad);

-- Trigger
CREATE TRIGGER update_vehiculos_updated_at
    BEFORE UPDATE ON vehiculos
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
