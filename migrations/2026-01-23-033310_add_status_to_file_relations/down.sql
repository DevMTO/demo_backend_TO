-- Revertir: quitar status de las tablas de relación de files (excepto files y file_tours que ya lo tenían)

-- 1) Quitar status de file_pasajeros
DROP INDEX IF EXISTS idx_file_pasajeros_status;
ALTER TABLE file_pasajeros DROP COLUMN IF EXISTS status;

-- 2) Quitar status de file_guias
DROP INDEX IF EXISTS idx_file_guias_status;
ALTER TABLE file_guias DROP COLUMN IF EXISTS status;

-- 3) Quitar status de file_restaurantes
DROP INDEX IF EXISTS idx_file_restaurantes_status;
ALTER TABLE file_restaurantes DROP COLUMN IF EXISTS status;

-- 4) Quitar status de file_vehiculos
DROP INDEX IF EXISTS idx_file_vehiculos_status;
ALTER TABLE file_vehiculos DROP COLUMN IF EXISTS status;

-- 5) Quitar status de file_entradas
DROP INDEX IF EXISTS idx_file_entradas_status;
ALTER TABLE file_entradas DROP COLUMN IF EXISTS status;

