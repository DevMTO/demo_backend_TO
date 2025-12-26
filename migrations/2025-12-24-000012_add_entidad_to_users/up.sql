-- Migration: Add id_entidad and nombre_entidad to users table
-- Permite asociar usuarios a entidades (agencia, transporte, etc.)

ALTER TABLE users 
    ADD COLUMN id_entidad UUID,
    ADD COLUMN tipo_entidad VARCHAR(30),
    ADD COLUMN nombre_entidad VARCHAR(200);

-- Constraint para tipo_entidad
ALTER TABLE users ADD CONSTRAINT chk_users_tipo_entidad CHECK (
    tipo_entidad IS NULL OR tipo_entidad IN ('agencia', 'transporte', 'restaurante', 'independiente')
);

-- Index
CREATE INDEX idx_users_id_entidad ON users(id_entidad);
CREATE INDEX idx_users_tipo_entidad ON users(tipo_entidad);

COMMENT ON COLUMN users.id_entidad IS 'UUID de la entidad asociada (agencia, transporte, restaurante)';
COMMENT ON COLUMN users.tipo_entidad IS 'Tipo de entidad: agencia, transporte, restaurante, independiente';
COMMENT ON COLUMN users.nombre_entidad IS 'Nombre de la entidad para display (desnormalizado para queries rápidos)';
