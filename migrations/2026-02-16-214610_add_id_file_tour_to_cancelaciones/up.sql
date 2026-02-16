-- Add id_file_tour to cancelaciones for per-tour cancellation support
ALTER TABLE cancelaciones ADD COLUMN id_file_tour INTEGER REFERENCES file_tours(id);

-- Drop old unique constraint (one cancelation per file)
ALTER TABLE cancelaciones DROP CONSTRAINT IF EXISTS uq_cancelacion_file;

-- New unique constraints:
-- 1. Only one file-level cancelation per file (where id_file_tour IS NULL)
CREATE UNIQUE INDEX uq_cancelacion_file_level ON cancelaciones (id_file) WHERE id_file_tour IS NULL;
-- 2. Only one tour-level cancelation per file_tour
CREATE UNIQUE INDEX uq_cancelacion_file_tour ON cancelaciones (id_file_tour) WHERE id_file_tour IS NOT NULL;

-- Add id_file_tour to no_shows for per-tour no-show support
ALTER TABLE no_shows ADD COLUMN id_file_tour INTEGER REFERENCES file_tours(id);

-- Drop old unique constraints
ALTER TABLE no_shows DROP CONSTRAINT IF EXISTS uq_no_show_file;

-- New unique constraints:
CREATE UNIQUE INDEX uq_no_show_file_level ON no_shows (id_file) WHERE id_file_tour IS NULL;
CREATE UNIQUE INDEX uq_no_show_file_tour ON no_shows (id_file_tour) WHERE id_file_tour IS NOT NULL;
