-- ========================================================================
-- REVERTIR OPTIMIZACIÓN DE TABLA USERS Y FILES
-- ========================================================================

-- ========================================================================
-- 1. REVERTIR CAMBIOS EN FILES
-- ========================================================================

-- 1.1 Quitar índice de nro_pasajeros
DROP INDEX IF EXISTS idx_files_nro_pasajeros;

-- 1.2 Quitar columna nro_pasajeros
ALTER TABLE files DROP COLUMN IF EXISTS nro_pasajeros;

-- ========================================================================
-- 2. REVERTIR CAMBIOS EN USERS
-- ========================================================================

-- 2.1 Quitar índice de is_active
DROP INDEX IF EXISTS idx_users_is_active;

-- 2.2 Restaurar columna nombre_entidad
ALTER TABLE users ADD COLUMN nombre_entidad VARCHAR(200);

-- 2.3 Restaurar columna status
ALTER TABLE users ADD COLUMN status VARCHAR(30) NOT NULL DEFAULT 'activo';

-- 2.4 Migrar datos de is_active a status
UPDATE users SET status = CASE 
    WHEN is_active = TRUE THEN 'activo' 
    ELSE 'inactivo' 
END;

-- 2.5 Quitar columna is_active
ALTER TABLE users DROP COLUMN is_active;

-- 2.6 Revertir default de role a 'agencia' (singular)
ALTER TABLE users ALTER COLUMN role SET DEFAULT 'agencia';

-- 2.7 Revertir roles al formato singular
UPDATE users SET role = 'agencia' WHERE role = 'agencias';
UPDATE users SET role = 'transporte' WHERE role = 'transportes';
UPDATE users SET role = 'conductor' WHERE role = 'conductores';
UPDATE users SET role = 'guia' WHERE role = 'guias';
UPDATE users SET role = 'restaurante' WHERE role = 'restaurantes';

-- 2.8 Recrear índice de status
CREATE INDEX idx_users_status ON users(status);

