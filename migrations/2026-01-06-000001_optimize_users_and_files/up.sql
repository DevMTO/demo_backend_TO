-- ========================================================================
-- OPTIMIZACIÓN DE TABLA USERS Y FILES
-- 1. Eliminar nombre_entidad de users (redundante con role)
-- 2. Cambiar status a is_active (booleano) en users
-- 3. Cambiar default de role de 'agencia' a 'agencias'
-- 4. Actualizar roles a formato plural (agencias, transportes, conductores, guias, restaurantes)
-- 5. Agregar nro_pasajeros a files
-- ========================================================================

-- ========================================================================
-- 1. CAMBIOS EN TABLA USERS
-- ========================================================================

-- 1.1 Agregar nueva columna is_active
ALTER TABLE users ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;

-- 1.2 Migrar datos de status a is_active
UPDATE users SET is_active = CASE 
    WHEN status IN ('activo', 'active') THEN TRUE 
    ELSE FALSE 
END;

-- 1.3 Eliminar columna status e índice asociado
DROP INDEX IF EXISTS idx_users_status;
ALTER TABLE users DROP COLUMN status;

-- 1.4 Eliminar columna nombre_entidad
ALTER TABLE users DROP COLUMN nombre_entidad;

-- 1.5 Cambiar default de role a 'agencias' (plural para consistencia)
ALTER TABLE users ALTER COLUMN role SET DEFAULT 'agencias';

-- 1.6 Actualizar registros existentes con roles al formato plural
UPDATE users SET role = 'agencias' WHERE role = 'agencia';
UPDATE users SET role = 'transportes' WHERE role = 'transporte';
UPDATE users SET role = 'conductores' WHERE role = 'conductor';
UPDATE users SET role = 'guias' WHERE role = 'guia';
UPDATE users SET role = 'restaurantes' WHERE role = 'restaurante';

-- 1.7 Crear índice para is_active
CREATE INDEX idx_users_is_active ON users(is_active) WHERE is_active = TRUE;

-- ========================================================================
-- 2. CAMBIOS EN TABLA FILES
-- ========================================================================

-- 2.1 Agregar columna nro_pasajeros
ALTER TABLE files ADD COLUMN nro_pasajeros INTEGER NOT NULL DEFAULT 0;

-- 2.2 Crear índice para nro_pasajeros
CREATE INDEX idx_files_nro_pasajeros ON files(nro_pasajeros) WHERE nro_pasajeros > 0;

