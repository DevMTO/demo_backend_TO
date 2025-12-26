-- ========================================================================
-- TABLA FILES
-- Files de viaje - reservaciones que vinculan tour con agencia
-- ========================================================================

CREATE TABLE files (
    id SERIAL PRIMARY KEY,
    id_tour INTEGER NOT NULL REFERENCES tours(id) ON DELETE RESTRICT,
    id_agencia INTEGER NOT NULL REFERENCES agencias(id) ON DELETE RESTRICT,
    
    -- Fechas y logística
    fecha_inicio DATE NOT NULL,
    fecha_fin DATE NOT NULL,
    lugar_recojo VARCHAR(200),
    hora_recojo TIME,
    notas TEXT,
    
    -- Estado y financiero
    status VARCHAR(30) NOT NULL DEFAULT 'pendiente',
    monto_total DECIMAL(12,2) NOT NULL DEFAULT 0,
    monto_pagado DECIMAL(12,2) NOT NULL DEFAULT 0,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    updated_by INTEGER REFERENCES users(id) ON DELETE SET NULL
);

-- Índices para búsquedas frecuentes
CREATE INDEX idx_files_id_tour ON files(id_tour);
CREATE INDEX idx_files_id_agencia ON files(id_agencia);
CREATE INDEX idx_files_status ON files(status);
CREATE INDEX idx_files_fecha_inicio ON files(fecha_inicio);
CREATE INDEX idx_files_created_at ON files(created_at DESC);
-- Índice compuesto para dashboard de agencia
CREATE INDEX idx_files_agencia_status_fecha ON files(id_agencia, status, fecha_inicio);
-- Índice para files pendientes de pago
CREATE INDEX idx_files_pendientes_pago ON files(status, monto_total, monto_pagado) 
    WHERE monto_pagado < monto_total;

-- Trigger para updated_at
CREATE TRIGGER update_files_updated_at
    BEFORE UPDATE ON files
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
