-- ========================================================================
-- AGREGAR CAMPO estado_confirmacion a file_guias y file_vehiculos
-- Para el flujo de confirmación de asignaciones
-- ========================================================================

-- Estados posibles: 'pendiente', 'aceptado', 'rechazado'

-- Agregar a file_guias
ALTER TABLE file_guias 
    ADD COLUMN estado_confirmacion VARCHAR(20) NOT NULL DEFAULT 'pendiente',
    ADD COLUMN confirmado_at TIMESTAMPTZ NULL,
    ADD COLUMN motivo_rechazo TEXT NULL;

-- Agregar a file_vehiculos (para el conductor)
ALTER TABLE file_vehiculos 
    ADD COLUMN estado_confirmacion VARCHAR(20) NOT NULL DEFAULT 'pendiente',
    ADD COLUMN confirmado_at TIMESTAMPTZ NULL,
    ADD COLUMN motivo_rechazo TEXT NULL;

-- Índices para filtrar por estado
CREATE INDEX idx_file_guias_estado ON file_guias(estado_confirmacion);
CREATE INDEX idx_file_vehiculos_estado ON file_vehiculos(estado_confirmacion);

-- Constraint para asegurar valores válidos
ALTER TABLE file_guias 
    ADD CONSTRAINT chk_file_guias_estado 
    CHECK (estado_confirmacion IN ('pendiente', 'aceptado', 'rechazado'));

ALTER TABLE file_vehiculos 
    ADD CONSTRAINT chk_file_vehiculos_estado 
    CHECK (estado_confirmacion IN ('pendiente', 'aceptado', 'rechazado'));
