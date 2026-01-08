-- Remove remember_me column from user_sessions
ALTER TABLE user_sessions DROP COLUMN IF EXISTS remember_me;
