-- Migration: Add last_activity_at to user_sessions
-- Para tracking de sesiones idle y rotación de tokens

ALTER TABLE user_sessions 
ADD COLUMN last_activity_at TIMESTAMPTZ;

-- Inicializar con created_at para sesiones existentes
UPDATE user_sessions SET last_activity_at = created_at WHERE last_activity_at IS NULL;

-- Index para búsquedas por actividad
CREATE INDEX idx_user_sessions_last_activity ON user_sessions(last_activity_at);

COMMENT ON COLUMN user_sessions.last_activity_at IS 'Timestamp of last user activity for idle timeout tracking';
