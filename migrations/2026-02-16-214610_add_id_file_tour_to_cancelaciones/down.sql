-- Remove partial indexes
DROP INDEX IF EXISTS uq_cancelacion_file_level;
DROP INDEX IF EXISTS uq_cancelacion_file_tour;
DROP INDEX IF EXISTS uq_no_show_file_level;
DROP INDEX IF EXISTS uq_no_show_file_tour;

-- Restore original unique constraints
ALTER TABLE cancelaciones ADD CONSTRAINT uq_cancelacion_file UNIQUE (id_file);
ALTER TABLE no_shows ADD CONSTRAINT uq_no_show_file UNIQUE (id_file);

-- Remove new columns
ALTER TABLE cancelaciones DROP COLUMN IF EXISTS id_file_tour;
ALTER TABLE no_shows DROP COLUMN IF EXISTS id_file_tour;
