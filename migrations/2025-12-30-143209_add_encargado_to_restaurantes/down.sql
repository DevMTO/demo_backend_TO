-- Remove encargado field from restaurantes table
DROP INDEX IF EXISTS idx_restaurantes_encargado;
ALTER TABLE restaurantes DROP COLUMN IF EXISTS encargado;
