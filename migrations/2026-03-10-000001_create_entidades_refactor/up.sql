-- ========================================================================
-- MIGRACIÓN: entidades_refactor (sin tabla entidades)
-- Renombra id_agencia → id_entidad y agrega columna entidad VARCHAR
-- en files y pagos_files. La columna entidad indica la tabla origen:
-- 'agencias', 'hoteles', etc.
-- ========================================================================

-- 0. Agregar paleta_colores a cadenas_hoteleras
ALTER TABLE cadenas_hoteleras ADD COLUMN paleta_colores JSONB DEFAULT '{}';

-- 0b. Simplificar hoteles: quitar media y encargado (innecesarios)
DROP INDEX IF EXISTS idx_hoteles_encargado;
ALTER TABLE hoteles DROP COLUMN IF EXISTS media;
ALTER TABLE hoteles DROP COLUMN IF EXISTS encargado;

-- 1. files: renombrar id_agencia → id_entidad, agregar entidad
ALTER TABLE files DROP CONSTRAINT IF EXISTS files_id_agencia_fkey;
DROP INDEX IF EXISTS idx_files_id_agencia;
DROP INDEX IF EXISTS idx_files_agencia_status_fecha;

-- Restaurar NOT NULL (perdido en revert anterior)
ALTER TABLE files ALTER COLUMN id_agencia SET NOT NULL;
ALTER TABLE files RENAME COLUMN id_agencia TO id_entidad;
ALTER TABLE files ADD COLUMN entidad VARCHAR(50);

-- Poblar entidad para registros existentes (todos son agencias)
UPDATE files SET entidad = 'agencias' WHERE entidad IS NULL;

-- Índices
CREATE INDEX idx_files_id_entidad ON files(id_entidad);
CREATE INDEX idx_files_entidad_status_fecha ON files(id_entidad, entidad, status, fecha_inicio);

-- 2. pagos_files: renombrar id_agencia → id_entidad, agregar entidad
ALTER TABLE pagos_files DROP CONSTRAINT IF EXISTS pagos_files_id_agencia_fkey;
DROP INDEX IF EXISTS idx_pagos_files_id_agencia;

-- Restaurar NOT NULL (perdido en revert anterior)
ALTER TABLE pagos_files ALTER COLUMN id_agencia SET NOT NULL;
ALTER TABLE pagos_files RENAME COLUMN id_agencia TO id_entidad;
ALTER TABLE pagos_files ADD COLUMN entidad VARCHAR(50);

-- Poblar entidad para registros existentes
UPDATE pagos_files SET entidad = 'agencias' WHERE entidad IS NULL;

-- Índices
CREATE INDEX idx_pagos_files_id_entidad ON pagos_files(id_entidad);
