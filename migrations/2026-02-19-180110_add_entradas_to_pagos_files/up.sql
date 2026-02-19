-- ========================================================================
-- Agregar campos entradas y entrada_precio a pagos_files
-- ========================================================================
-- entradas (BOOLEAN): indica si este registro de pago cubre entradas
-- entrada_precio (NUMERIC): costo de las entradas del file_tour asociado
--   Solo tiene valor cuando entradas = true, de lo contrario es NULL
-- ========================================================================

ALTER TABLE pagos_files
    ADD COLUMN IF NOT EXISTS entradas BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS entrada_precio NUMERIC(12,2) DEFAULT NULL;

-- Ampliar el estado a 30 chars para dar margen
ALTER TABLE pagos_files ALTER COLUMN estado TYPE VARCHAR(30);
