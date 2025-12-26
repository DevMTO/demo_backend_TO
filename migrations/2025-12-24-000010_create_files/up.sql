-- Migration: Create files table
-- Files de viaje (reservación de grupo/paquete turístico)

CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_code VARCHAR(20) NOT NULL UNIQUE,  -- Código único del file, ej: "F-2024-001"
    id_tour UUID NOT NULL REFERENCES tours(id) ON DELETE RESTRICT,
    id_agencia UUID NOT NULL REFERENCES agencias(id) ON DELETE RESTRICT,
    
    -- Asignaciones (IDs en arrays JSON para flexibilidad)
    guias JSONB DEFAULT '[]',           -- [{"id": "uuid", "nombre": "...", "rol": "principal/asistente"}]
    pasajeros JSONB DEFAULT '[]',       -- [{"id": "uuid", "nombre": "...", "asiento": "..."}]
    vehiculos JSONB DEFAULT '[]',       -- [{"id": "uuid", "placa": "...", "conductor_id": "uuid"}]
    restaurante JSONB DEFAULT '{}',     -- {"id": "uuid", "nombre": "...", "tipo_servicio": "almuerzo"}
    entradas JSONB DEFAULT '[]',        -- [{"id": "uuid", "cantidad": 10, "tipo": "turista"}]
    
    -- Fechas y logística
    fechas JSONB DEFAULT '{}',          -- {"inicio": "2024-01-15", "fin": "2024-01-17", "dias": ["..."]}
    lugar_recojo VARCHAR(200),
    hora_recojo TIMESTAMPTZ,
    notas TEXT,
    
    -- Estado y financiero
    status VARCHAR(30) NOT NULL DEFAULT 'pendiente',
    monto_total DECIMAL(12,2) NOT NULL DEFAULT 0,
    monto_pagado DECIMAL(12,2) NOT NULL DEFAULT 0,
    
    -- Auditoría
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT chk_files_status CHECK (
        status IN ('pendiente', 'confirmado', 'en_curso', 'completado', 'cancelado')
    ),
    CONSTRAINT chk_files_monto_total CHECK (monto_total >= 0),
    CONSTRAINT chk_files_monto_pagado CHECK (monto_pagado >= 0)
);

-- Indexes
CREATE INDEX idx_files_file_code ON files(file_code);
CREATE INDEX idx_files_id_tour ON files(id_tour);
CREATE INDEX idx_files_id_agencia ON files(id_agencia);
CREATE INDEX idx_files_status ON files(status);
CREATE INDEX idx_files_created_at ON files(created_at DESC);
CREATE INDEX idx_files_hora_recojo ON files(hora_recojo);

-- Trigger
CREATE TRIGGER update_files_updated_at
    BEFORE UPDATE ON files
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Función para generar código de file automáticamente
CREATE OR REPLACE FUNCTION generate_file_code()
RETURNS TRIGGER AS $$
DECLARE
    year_prefix TEXT;
    next_num INTEGER;
BEGIN
    year_prefix := 'F-' || EXTRACT(YEAR FROM NOW())::TEXT || '-';
    
    SELECT COALESCE(MAX(CAST(SUBSTRING(file_code FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO next_num
    FROM files
    WHERE file_code LIKE year_prefix || '%';
    
    NEW.file_code := year_prefix || LPAD(next_num::TEXT, 4, '0');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_generate_file_code
    BEFORE INSERT ON files
    FOR EACH ROW
    WHEN (NEW.file_code IS NULL OR NEW.file_code = '')
    EXECUTE FUNCTION generate_file_code();
