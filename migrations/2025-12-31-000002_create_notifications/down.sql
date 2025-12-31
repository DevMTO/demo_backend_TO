-- Eliminar índices de notification_users
DROP INDEX IF EXISTS idx_notification_users_unread;
DROP INDEX IF EXISTS idx_notification_users_notification;
DROP INDEX IF EXISTS idx_notification_users_user;

-- Eliminar índices de notifications
DROP INDEX IF EXISTS idx_notifications_target_roles;
DROP INDEX IF EXISTS idx_notifications_expires_at;
DROP INDEX IF EXISTS idx_notifications_created_at;
DROP INDEX IF EXISTS idx_notifications_target_user;
DROP INDEX IF EXISTS idx_notifications_priority;
DROP INDEX IF EXISTS idx_notifications_category;
DROP INDEX IF EXISTS idx_notifications_type;

-- Eliminar tablas en orden de dependencia
DROP TABLE IF EXISTS notification_users;
DROP TABLE IF EXISTS notifications;
