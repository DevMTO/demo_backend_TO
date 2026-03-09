-- ========================================================================
-- ROLLBACK: entidades_refactor
-- Restaura id_agencia en files y pagos_files, elimina entidad
-- ========================================================================

-- 1. pagos_files: quitar entidad, renombrar id_entidad → id_agencia
DROP INDEX IF EXISTS idx_pagos_files_id_entidad;
ALTER TABLE pagos_files DROP COLUMN IF EXISTS entidad;
ALTER TABLE pagos_files RENAME COLUMN id_entidad TO id_agencia;
CREATE INDEX idx_pagos_files_id_agencia ON pagos_files(id_agencia);

-- 2. files: quitar entidad, renombrar id_entidad → id_agencia
DROP INDEX IF EXISTS idx_files_id_entidad;
DROP INDEX IF EXISTS idx_files_entidad_status_fecha;
ALTER TABLE files DROP COLUMN IF EXISTS entidad;
ALTER TABLE files RENAME COLUMN id_entidad TO id_agencia;
CREATE INDEX idx_files_id_agencia ON files(id_agencia);
CREATE INDEX idx_files_agencia_status_fecha ON files(id_agencia, status, fecha_inicio);

-- 3. Revertir paleta_colores de cadenas_hoteleras
ALTER TABLE cadenas_hoteleras DROP COLUMN IF EXISTS paleta_colores;

-- 4. Restaurar media y encargado en hoteles
ALTER TABLE hoteles ADD COLUMN media JSONB DEFAULT '{}';
ALTER TABLE hoteles ADD COLUMN encargado INTEGER REFERENCES personas(id) ON DELETE SET NULL;
CREATE INDEX idx_hoteles_encargado ON hoteles(encargado) WHERE encargado IS NOT NULL;
