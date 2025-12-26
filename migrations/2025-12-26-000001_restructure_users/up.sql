-- Migration: Restructure users table to match diagram
-- user: id, id_persona, username, email, password, role, id_entidad, nombre_entidad, status

-- PASO 1: Eliminar TODOS los constraints de role y status PRIMERO
ALTER TABLE users DROP CONSTRAINT IF EXISTS chk_users_role;
ALTER TABLE users DROP CONSTRAINT IF EXISTS chk_users_status;
ALTER TABLE users DROP CONSTRAINT IF EXISTS chk_users_tipo_entidad;

-- PASO 2: Eliminar columnas que ya no necesitamos
ALTER TABLE users DROP COLUMN IF EXISTS display_name;
ALTER TABLE users DROP COLUMN IF EXISTS email_verified;
ALTER TABLE users DROP COLUMN IF EXISTS is_active;
ALTER TABLE users DROP COLUMN IF EXISTS created_by;
ALTER TABLE users DROP COLUMN IF EXISTS updated_by;
ALTER TABLE users DROP COLUMN IF EXISTS version;
ALTER TABLE users DROP COLUMN IF EXISTS mfa_enabled;
ALTER TABLE users DROP COLUMN IF EXISTS mfa_secret;
ALTER TABLE users DROP COLUMN IF EXISTS mfa_backup_codes;
ALTER TABLE users DROP COLUMN IF EXISTS tipo_entidad;

-- PASO 3: Agregar nuevas columnas
ALTER TABLE users ADD COLUMN IF NOT EXISTS id_persona UUID;
ALTER TABLE users ADD COLUMN IF NOT EXISTS status VARCHAR(30) DEFAULT 'activo';

-- PASO 4: Actualizar status donde sea NULL
UPDATE users SET status = 'activo' WHERE status IS NULL;

-- PASO 5: Hacer status NOT NULL ahora que tiene datos
ALTER TABLE users ALTER COLUMN status SET NOT NULL;

-- PASO 6: Actualizar roles existentes
UPDATE users SET role = 'operador' WHERE role = 'user';
UPDATE users SET role = 'operador' WHERE role IS NULL;
UPDATE users SET role = 'superadmin' WHERE role NOT IN ('superadmin', 'admin', 'subadmin', 'operador', 'viewer');

-- PASO 7: Crear nuevos constraints
ALTER TABLE users ADD CONSTRAINT chk_users_status CHECK (
    status IN ('activo', 'inactivo', 'suspendido', 'pendiente_verificacion')
);

ALTER TABLE users ADD CONSTRAINT chk_users_role CHECK (
    role IN ('superadmin', 'admin', 'subadmin', 'operador', 'viewer')
);

-- PASO 8: Crear índices
CREATE INDEX IF NOT EXISTS idx_users_id_persona ON users(id_persona);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);

COMMENT ON COLUMN users.id_persona IS 'FK a la tabla personas - datos personales del usuario';
COMMENT ON COLUMN users.status IS 'Estado del usuario: activo, inactivo, suspendido, pendiente_verificacion';
