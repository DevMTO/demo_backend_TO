-- Cleanup Functions for Activity Logs and Notifications
-- Estas funciones permiten limpieza automática de registros antiguos

-- ============================================================
-- Función para limpiar activity logs antiguos
-- Mantiene logs de los últimos N días según el tipo
-- ============================================================
CREATE OR REPLACE FUNCTION cleanup_old_activity_logs(
    days_to_keep_success INTEGER DEFAULT 30,
    days_to_keep_error INTEGER DEFAULT 90,
    days_to_keep_auth INTEGER DEFAULT 60
) RETURNS TABLE(
    deleted_success BIGINT,
    deleted_error BIGINT,
    deleted_auth BIGINT,
    total_deleted BIGINT
) AS $$
DECLARE
    v_deleted_success BIGINT := 0;
    v_deleted_error BIGINT := 0;
    v_deleted_auth BIGINT := 0;
BEGIN
    -- Eliminar logs de success antiguos
    WITH deleted AS (
        DELETE FROM activity_logs 
        WHERE status = 'success' 
        AND action_type != 'AUTH'
        AND created_at < CURRENT_TIMESTAMP - (days_to_keep_success || ' days')::INTERVAL
        RETURNING 1
    )
    SELECT COUNT(*) INTO v_deleted_success FROM deleted;
    
    -- Eliminar logs de error antiguos (mantener más tiempo para auditoría)
    WITH deleted AS (
        DELETE FROM activity_logs 
        WHERE status = 'error'
        AND created_at < CURRENT_TIMESTAMP - (days_to_keep_error || ' days')::INTERVAL
        RETURNING 1
    )
    SELECT COUNT(*) INTO v_deleted_error FROM deleted;
    
    -- Eliminar logs de autenticación (login/logout) antiguos
    WITH deleted AS (
        DELETE FROM activity_logs 
        WHERE action_type = 'AUTH'
        AND created_at < CURRENT_TIMESTAMP - (days_to_keep_auth || ' days')::INTERVAL
        RETURNING 1
    )
    SELECT COUNT(*) INTO v_deleted_auth FROM deleted;
    
    RETURN QUERY SELECT 
        v_deleted_success,
        v_deleted_error,
        v_deleted_auth,
        v_deleted_success + v_deleted_error + v_deleted_auth;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- Función para limpiar notificaciones expiradas y antiguas
-- Respeta la prioridad para decidir cuánto tiempo mantener
-- ============================================================
CREATE OR REPLACE FUNCTION cleanup_notifications(
    days_low_priority INTEGER DEFAULT 7,      -- Low: 7 días
    days_normal_priority INTEGER DEFAULT 14,  -- Normal: 14 días  
    days_high_priority INTEGER DEFAULT 30,    -- High: 30 días
    days_urgent_priority INTEGER DEFAULT 60   -- Urgent: 60 días
) RETURNS TABLE(
    deleted_expired BIGINT,
    deleted_low BIGINT,
    deleted_normal BIGINT,
    deleted_high BIGINT,
    deleted_urgent BIGINT,
    total_deleted BIGINT
) AS $$
DECLARE
    v_deleted_expired BIGINT := 0;
    v_deleted_low BIGINT := 0;
    v_deleted_normal BIGINT := 0;
    v_deleted_high BIGINT := 0;
    v_deleted_urgent BIGINT := 0;
BEGIN
    -- Primero, eliminar notificaciones expiradas (sin importar prioridad)
    WITH deleted AS (
        DELETE FROM notifications 
        WHERE expires_at IS NOT NULL 
        AND expires_at < CURRENT_TIMESTAMP
        RETURNING 1
    )
    SELECT COUNT(*) INTO v_deleted_expired FROM deleted;
    
    -- Eliminar notificaciones de prioridad LOW antiguas
    WITH deleted AS (
        DELETE FROM notifications 
        WHERE priority = 'low'
        AND expires_at IS NULL
        AND created_at < CURRENT_TIMESTAMP - (days_low_priority || ' days')::INTERVAL
        RETURNING 1
    )
    SELECT COUNT(*) INTO v_deleted_low FROM deleted;
    
    -- Eliminar notificaciones de prioridad NORMAL antiguas
    WITH deleted AS (
        DELETE FROM notifications 
        WHERE priority = 'normal'
        AND expires_at IS NULL
        AND created_at < CURRENT_TIMESTAMP - (days_normal_priority || ' days')::INTERVAL
        RETURNING 1
    )
    SELECT COUNT(*) INTO v_deleted_normal FROM deleted;
    
    -- Eliminar notificaciones de prioridad HIGH antiguas
    WITH deleted AS (
        DELETE FROM notifications 
        WHERE priority = 'high'
        AND expires_at IS NULL
        AND created_at < CURRENT_TIMESTAMP - (days_high_priority || ' days')::INTERVAL
        RETURNING 1
    )
    SELECT COUNT(*) INTO v_deleted_high FROM deleted;
    
    -- Eliminar notificaciones de prioridad URGENT antiguas
    WITH deleted AS (
        DELETE FROM notifications 
        WHERE priority = 'urgent'
        AND expires_at IS NULL
        AND created_at < CURRENT_TIMESTAMP - (days_urgent_priority || ' days')::INTERVAL
        RETURNING 1
    )
    SELECT COUNT(*) INTO v_deleted_urgent FROM deleted;
    
    RETURN QUERY SELECT 
        v_deleted_expired,
        v_deleted_low,
        v_deleted_normal,
        v_deleted_high,
        v_deleted_urgent,
        v_deleted_expired + v_deleted_low + v_deleted_normal + v_deleted_high + v_deleted_urgent;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- Función para limpiar notification_users huérfanos
-- (cuando la notificación ya no existe)
-- ============================================================
CREATE OR REPLACE FUNCTION cleanup_orphan_notification_users() 
RETURNS BIGINT AS $$
DECLARE
    deleted_count BIGINT := 0;
BEGIN
    WITH deleted AS (
        DELETE FROM notification_users nu
        WHERE NOT EXISTS (
            SELECT 1 FROM notifications n WHERE n.id = nu.notification_id
        )
        RETURNING 1
    )
    SELECT COUNT(*) INTO deleted_count FROM deleted;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- Vista para estadísticas de logs
-- ============================================================
CREATE OR REPLACE VIEW activity_logs_stats AS
SELECT 
    action_type,
    status,
    COUNT(*) as count,
    MIN(created_at) as oldest,
    MAX(created_at) as newest
FROM activity_logs
GROUP BY action_type, status
ORDER BY action_type, status;

-- ============================================================
-- Vista para estadísticas de notificaciones
-- ============================================================
CREATE OR REPLACE VIEW notifications_stats AS
SELECT 
    notification_type,
    category,
    priority,
    COUNT(*) as count,
    COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at < CURRENT_TIMESTAMP) as expired_count,
    MIN(created_at) as oldest,
    MAX(created_at) as newest
FROM notifications
GROUP BY notification_type, category, priority
ORDER BY priority, category, notification_type;

-- ============================================================
-- Índice parcial para notificaciones no leídas (optimización)
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_notification_users_pending 
ON notification_users(user_id, notification_id) 
WHERE is_read = FALSE AND is_dismissed = FALSE;

-- Comentarios para documentación
COMMENT ON FUNCTION cleanup_old_activity_logs IS 
'Limpia activity logs antiguos según configuración de días. 
 Params: days_to_keep_success (30), days_to_keep_error (90), days_to_keep_auth (60)';

COMMENT ON FUNCTION cleanup_notifications IS 
'Limpia notificaciones expiradas y antiguas según prioridad.
 Low: 7 días, Normal: 14 días, High: 30 días, Urgent: 60 días';

COMMENT ON FUNCTION cleanup_orphan_notification_users IS 
'Elimina registros de notification_users huérfanos (sin notificación padre)';
