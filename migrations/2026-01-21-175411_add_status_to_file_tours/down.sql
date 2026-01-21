-- Revertir: quitar status de file_tours
ALTER TABLE file_tours DROP COLUMN IF EXISTS status;
