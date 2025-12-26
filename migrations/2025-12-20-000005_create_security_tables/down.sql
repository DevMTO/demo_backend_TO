-- Revert: Drop security tables
DROP TRIGGER IF EXISTS update_oauth_providers_updated_at ON oauth_providers;
DROP INDEX IF EXISTS idx_oauth_providers_provider;
DROP INDEX IF EXISTS idx_oauth_providers_user_id;
DROP TABLE IF EXISTS oauth_providers;

DROP INDEX IF EXISTS idx_refresh_tokens_expires;
DROP INDEX IF EXISTS idx_refresh_tokens_token_hash;
DROP INDEX IF EXISTS idx_refresh_tokens_session_id;
DROP INDEX IF EXISTS idx_refresh_tokens_user_id;
DROP TABLE IF EXISTS refresh_tokens;

DROP INDEX IF EXISTS idx_login_attempts_failed;
DROP INDEX IF EXISTS idx_login_attempts_created_at;
DROP INDEX IF EXISTS idx_login_attempts_ip;
DROP INDEX IF EXISTS idx_login_attempts_identifier;
DROP TABLE IF EXISTS login_attempts;
