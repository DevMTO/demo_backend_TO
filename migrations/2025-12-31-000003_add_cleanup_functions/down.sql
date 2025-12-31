-- Revertir funciones y vistas de cleanup

DROP VIEW IF EXISTS notifications_stats;
DROP VIEW IF EXISTS activity_logs_stats;
DROP FUNCTION IF EXISTS cleanup_orphan_notification_users();
DROP FUNCTION IF EXISTS cleanup_notifications(INTEGER, INTEGER, INTEGER, INTEGER);
DROP FUNCTION IF EXISTS cleanup_old_activity_logs(INTEGER, INTEGER, INTEGER);
DROP INDEX IF EXISTS idx_notification_users_pending;
