-- Eliminar índices
DROP INDEX IF EXISTS idx_activity_logs_changed_fields;
DROP INDEX IF EXISTS idx_activity_logs_new_values;
DROP INDEX IF EXISTS idx_activity_logs_status;
DROP INDEX IF EXISTS idx_activity_logs_created_at;
DROP INDEX IF EXISTS idx_activity_logs_entity;
DROP INDEX IF EXISTS idx_activity_logs_action_type;
DROP INDEX IF EXISTS idx_activity_logs_user_id;

-- Eliminar tabla
DROP TABLE IF EXISTS activity_logs;
