-- Rollback: Remove entidad columns from users
ALTER TABLE users DROP CONSTRAINT IF EXISTS chk_users_tipo_entidad;
ALTER TABLE users DROP COLUMN IF EXISTS nombre_entidad;
ALTER TABLE users DROP COLUMN IF EXISTS tipo_entidad;
ALTER TABLE users DROP COLUMN IF EXISTS id_entidad;
