-- Revertir file_tours.notas de JSONB a TEXT
ALTER TABLE file_tours
  ALTER COLUMN notas TYPE TEXT
  USING notas::text;

