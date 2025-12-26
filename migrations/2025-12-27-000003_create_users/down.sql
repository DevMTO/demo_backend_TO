ALTER TABLE personas DROP CONSTRAINT IF EXISTS fk_personas_created_by;
ALTER TABLE personas DROP CONSTRAINT IF EXISTS fk_personas_updated_by;
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
DROP TABLE IF EXISTS users CASCADE;
