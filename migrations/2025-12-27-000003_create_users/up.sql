-- ========================================================================
-- TABLA USERS
-- Usuarios del sistema con autenticación
-- ========================================================================

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    id_persona INTEGER REFERENCES personas(id) ON DELETE SET NULL,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'operador',
    id_entidad INTEGER,
    nombre_entidad VARCHAR(200),
    status VARCHAR(30) NOT NULL DEFAULT 'activo',
    last_login TIMESTAMPTZ,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER,
    updated_by INTEGER
);

-- Índices para autenticación y búsquedas
CREATE UNIQUE INDEX idx_users_email ON users(email);
CREATE UNIQUE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_id_persona ON users(id_persona) WHERE id_persona IS NOT NULL;
CREATE INDEX idx_users_id_entidad ON users(id_entidad) WHERE id_entidad IS NOT NULL;
CREATE INDEX idx_users_created_at ON users(created_at DESC);

-- Trigger para updated_at
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Agregar FKs de auditoría a personas
ALTER TABLE personas 
    ADD CONSTRAINT fk_personas_created_by FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL,
    ADD CONSTRAINT fk_personas_updated_by FOREIGN KEY (updated_by) REFERENCES users(id) ON DELETE SET NULL;

-- Self-reference FKs para users
ALTER TABLE users
    ADD CONSTRAINT fk_users_created_by FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL,
    ADD CONSTRAINT fk_users_updated_by FOREIGN KEY (updated_by) REFERENCES users(id) ON DELETE SET NULL;
