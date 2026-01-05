-- Remove paleta_colores column from transportes table
ALTER TABLE transportes
DROP COLUMN IF EXISTS paleta_colores;
