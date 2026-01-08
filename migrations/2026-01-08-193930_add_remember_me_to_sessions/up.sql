-- Add remember_me column to user_sessions
-- When true, the session will not expire due to inactivity (idle timeout)
-- but will still expire at expires_at (absolute expiration)

ALTER TABLE user_sessions 
ADD COLUMN remember_me BOOLEAN NOT NULL DEFAULT FALSE;

-- Comment for documentation
COMMENT ON COLUMN user_sessions.remember_me IS 'If true, session does not expire by idle timeout';
