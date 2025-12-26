-- ========================================================================
-- TABLA USER_SESSIONS
-- Sesiones activas de usuarios
-- ========================================================================

CREATE TABLE user_sessions (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    refresh_token_hash VARCHAR(255),
    expires_at TIMESTAMPTZ NOT NULL,
    refresh_expires_at TIMESTAMPTZ,
    ip_address VARCHAR(45),
    user_agent TEXT,
    device_fingerprint VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_activity TIMESTAMPTZ DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,
    revoked_reason VARCHAR(50),
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índices para validación de sesiones (crítico para performance)
CREATE INDEX idx_user_sessions_token_hash ON user_sessions(token_hash);
CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_active_expires ON user_sessions(is_active, expires_at) WHERE is_active = TRUE;
CREATE INDEX idx_user_sessions_refresh_token ON user_sessions(refresh_token_hash) WHERE refresh_token_hash IS NOT NULL;
CREATE INDEX idx_user_sessions_last_activity ON user_sessions(last_activity DESC);

-- Trigger para updated_at
CREATE TRIGGER update_user_sessions_updated_at
    BEFORE UPDATE ON user_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
