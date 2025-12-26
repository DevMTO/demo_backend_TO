-- Rollback: Drop personas table

ALTER TABLE users DROP COLUMN IF EXISTS id_persona;
DROP TABLE IF EXISTS personas;
