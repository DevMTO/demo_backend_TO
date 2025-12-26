-- Revert: Remove last_activity_at from user_sessions

DROP INDEX IF EXISTS idx_user_sessions_last_activity;

ALTER TABLE user_sessions 
DROP COLUMN IF EXISTS last_activity_at;
